use winit::dpi::PhysicalSize;

use super::{WindowDescriptor, WindowId};

pub struct CreateWindow {
    pub id: WindowId,
    pub desc: WindowDescriptor,
}

pub struct WindowCreated {
    pub id: WindowId,
}

pub struct CloseWindow {
    pub id: WindowId,
}

pub struct WindowClosed {
    pub id: WindowId,
}

pub struct WindowResized {
    pub id: WindowId,
    pub new_size: PhysicalSize<u32>,
}

pub struct RequestRedraw;

pub struct FocusChanged {
    pub window_id: WindowId,
    pub focused: bool,
}

pub struct CursorEntered {
    pub window_id: WindowId,
}

pub struct CursorLeft {
    pub window_id: WindowId,
}
