use std::collections::HashMap;

use bevy_app::{CoreStage, Plugin};
use bevy_ecs::system::IntoExclusiveSystem;
use winit::{
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

use self::{
    commands::WindowCommands,
    events::{CreateWindow, CursorEntered, CursorLeft, FocusChanged, RequestRedraw, WindowCreated},
    runner::{execute_window_commands, handle_create_window, winit_event_loop_runner},
};

pub mod commands;
pub mod events;
pub mod runner;
pub mod util;

pub struct FlatWinitPlugin {
    pub create_primary_window: bool,
}

impl Default for FlatWinitPlugin {
    fn default() -> Self {
        Self {
            create_primary_window: true,
        }
    }
}

impl Plugin for FlatWinitPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let event_loop = EventLoop::new();

        app.init_resource::<WinitWindows>()
            .set_runner(winit_event_loop_runner)
            // NOTE: What is ExclusiveSystem
            .add_system_to_stage(
                CoreStage::PostUpdate,
                execute_window_commands.exclusive_system(),
            );

        if self.create_primary_window {
            app.world.init_resource::<WindowDescriptor>();
            let desc = app
                .world
                .get_resource::<WindowDescriptor>()
                .cloned()
                .unwrap();
            app.world.send_event(CreateWindow {
                id: WindowId::primary(),
                desc,
            });
        }
        handle_create_window(&mut app.world, &event_loop);

        app.insert_non_send_resource(event_loop);
    }
}

pub struct FlatWindowPlugin;
impl Plugin for FlatWindowPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Windows>()
            .add_event::<CreateWindow>()
            .add_event::<WindowCreated>()
            .add_event::<RequestRedraw>()
            .add_event::<FocusChanged>()
            .add_event::<CursorEntered>()
            .add_event::<CursorLeft>();
    }
}

#[derive(Default)]
pub struct WinitWindows {
    map: HashMap<WindowId, winit::window::Window>,
    winit_to_lib: HashMap<winit::window::WindowId, WindowId>,
    lib_to_winit: HashMap<WindowId, winit::window::WindowId>,
}

impl WinitWindows {
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        id: WindowId,
        desc: WindowDescriptor,
    ) -> Window {
        let builder = WindowBuilder::new();

        // TODO: build window from desc
        //
        //

        let winit_window = builder.build(event_loop).expect("Window build failed");

        self.winit_to_lib.insert(winit_window.id(), id);
        self.lib_to_winit.insert(id, winit_window.id());
        self.map.insert(id, winit_window);

        Window::new(id, desc)
    }
}

pub struct Windows {
    pub map: HashMap<WindowId, Window>,
    next_id: usize,
}

impl Default for Windows {
    fn default() -> Self {
        Self {
            map: Default::default(),
            next_id: 1,
        }
    }
}

impl Windows {
    pub fn add(&mut self, window: Window) {
        self.map.insert(window.id, window);
    }

    pub fn reserve_id(&mut self) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        id
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WindowId(pub usize);
impl WindowId {
    const PRIMARY_ID: usize = 0;

    pub fn new(id: usize) -> Self {
        assert_ne!(id, 0);
        Self(id)
    }

    pub fn primary() -> Self {
        Self(Self::PRIMARY_ID)
    }

    pub fn is_primary(&self) -> bool {
        self.0 == Self::PRIMARY_ID
    }
}

pub struct Window {
    pub id: WindowId,
    pub desc: WindowDescriptor,
    command_queue: Vec<WindowCommands>,
}

impl Window {
    pub fn new(id: WindowId, desc: WindowDescriptor) -> Self {
        Self {
            id,
            desc,
            command_queue: Vec::new(),
        }
    }

    pub fn execute(&mut self, command: WindowCommands) {
        self.command_queue.push(command);
    }
}

#[derive(Clone)]
pub struct WindowDescriptor {}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {}
    }
}
