use std::{thread::{JoinHandle, self}, sync::Arc, future::Future, pin::Pin, task::{Context, Poll}, path::Path};

use async_executor::Executor;
use crossbeam_channel::TryRecvError;
use futures_lite::future;

use super::{loader::FileAssetIo, lifecycle::AssetLifecycle};


/// Copied from bevy_tasks-0.7.0 - crate::task
pub struct Task<T>(async_executor::Task<T>);

impl<T> Task<T> {
    /// Creates a new task from a given `async_executor::Task`
    pub fn new(task: async_executor::Task<T>) -> Self {
        Self(task)
    }

    pub fn detach(self) {
        self.0.detach();
    }

    pub async fn cancel(self) -> Option<T> {
        self.0.cancel().await
    }
}

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Learn what this is
        Pin::new(&mut self.0).poll(cx)
        // self.0.poll(cx)
    }
}

struct TaskPoolInner {
    /// Async executor threads which spin indefinitely
    /// and let tasks to be spawned and run in the background
    threads: Vec<JoinHandle<()>>,
    shutdown_tx: async_channel::Sender<()>,
}

impl Drop for TaskPoolInner {
    // When dropped, join all executor threads
    // by closing the shutdown_tx/rx channel
    fn drop(&mut self) {
        self.shutdown_tx.close();

        let panicking = thread::panicking();
        for join_handle in self.threads.drain(..) {
            let res = join_handle.join();
            if !panicking {
                res.expect("Task thread panicked while executing.");
            }
        }
    }
}

pub struct TaskPool {
    executor: Arc<Executor<'static>>,
    inner: TaskPoolInner,
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

impl TaskPool {
    pub fn new(
        num_threads: Option<usize>,
        stack_size: Option<usize>,
        thread_name: Option<&str>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = async_channel::unbounded::<()>();
        let executor = Arc::new(Executor::new());

        let num_threads = num_threads.unwrap_or_else(num_cpus::get);
    
        let threads = (0..num_threads)
            .map(|i| {
                let shutdown_rx = shutdown_rx.clone();
                let ex = Arc::clone(&executor);

                let mut thread_builder = thread::Builder::new()
                    .name(format!("{} - {}", thread_name.unwrap_or("TaskPoolWorker"), i));
                if let Some(stack_size) = stack_size {
                    thread_builder = thread_builder.stack_size(stack_size);
                }

                thread_builder
                    .spawn(move || {
                        let shutdown_future = 
                            ex.run(shutdown_rx.recv());
                        // Expect Closed Err
                        future::block_on(shutdown_future).unwrap_err();
                    })
                    .expect("Failed to spawn thread")
            })
            .collect();
    
        Self {
            executor,
            inner: TaskPoolInner {
                threads,
                shutdown_tx,
            }
        }
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Task::new(self.executor.spawn(future))
    }
}
