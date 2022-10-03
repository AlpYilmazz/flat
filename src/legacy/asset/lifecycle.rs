use crossbeam_channel::{Sender, Receiver};



pub struct AssetLifecycle<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl<T> AssetLifecycle<T> {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self {
            sender,
            receiver,
        }
    }

    pub fn create(&self, asset: T) {
        self.sender.send(asset).expect("Sender Err");
    }
}