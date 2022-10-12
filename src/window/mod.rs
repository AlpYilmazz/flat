use std::collections::HashMap;

use ::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use bevy_app::{CoreStage, Plugin};
use bevy_ecs::system::IntoExclusiveSystem;
use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

use self::{
    commands::WindowCommands,
    events::{
        CreateWindow, CursorEntered, CursorLeft, FocusChanged, RequestRedraw, WindowCreated,
        WindowResized,
    },
    raw_window_handle::RawWindowHandleWrapper,
    runner::{
        execute_window_commands, handle_initial_create_window, winit_event_loop_runner,
        WinitSettings,
    },
};

pub mod commands;
pub mod events;
pub mod raw_window_handle;
pub mod runner;
pub mod util;

#[derive(Clone, Copy)]
pub enum ExitOnWindowClose {
    Any,
    Primary,
    All,
}

pub struct FlatWinitPlugin {
    pub create_primary_window: bool,
    pub exit_on_close: ExitOnWindowClose,
}

impl Default for FlatWinitPlugin {
    fn default() -> Self {
        Self {
            create_primary_window: true,
            exit_on_close: ExitOnWindowClose::Any,
        }
    }
}

impl Plugin for FlatWinitPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let event_loop = EventLoop::new();

        app.init_resource::<WinitWindows>()
            .insert_resource(WinitSettings {
                exit_on_close: self.exit_on_close,
                run_return: true,
            })
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
        handle_initial_create_window(&mut app.world, &event_loop);

        app.insert_non_send_resource(event_loop);
    }
}

pub struct FlatWindowPlugin;
impl Plugin for FlatWindowPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Windows>()
            .add_event::<CreateWindow>()
            .add_event::<WindowCreated>()
            .add_event::<WindowResized>()
            .add_event::<RequestRedraw>()
            .add_event::<FocusChanged>()
            .add_event::<CursorEntered>()
            .add_event::<CursorLeft>();
    }
}

#[derive(Default)]
pub struct WinitWindows {
    pub map: HashMap<WindowId, winit::window::Window>,
    winit_to_lib: HashMap<winit::window::WindowId, WindowId>,
    lib_to_winit: HashMap<WindowId, winit::window::WindowId>,
}

impl WinitWindows {
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        id: WindowId,
        desc: &WindowDescriptor,
    ) -> Window {
        let builder = WindowBuilder::new();

        // TODO: build window from desc
        //
        //

        let winit_window = builder.build(event_loop).expect("Window build failed");
        // winit_window.request_redraw();

        let scale_factor = winit_window.scale_factor();
        let physical_size = winit_window.inner_size();
        let raw_window_handle = winit_window.raw_window_handle();

        self.winit_to_lib.insert(winit_window.id(), id);
        self.lib_to_winit.insert(id, winit_window.id());
        self.map.insert(id, winit_window);

        Window::new(id, &desc, scale_factor, physical_size, raw_window_handle)
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
        assert_ne!(id, Self::PRIMARY_ID);
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
    pub scale_factor: f64,
    pub physical_size: PhysicalSize<u32>,
    pub raw_window_handle: RawWindowHandleWrapper,
    command_queue: Vec<WindowCommands>,
}

impl Window {
    pub fn new(
        id: WindowId,
        _desc: &WindowDescriptor,
        scale_factor: f64,
        physical_size: PhysicalSize<u32>,
        raw_window_handle: RawWindowHandle,
    ) -> Self {
        Self {
            id,
            scale_factor,
            physical_size,
            raw_window_handle: RawWindowHandleWrapper::new(raw_window_handle),
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
