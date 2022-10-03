
// NOTE: Copied from bevy_window-0.7.0

use cgmath::Vector2;

pub enum WindowCommands {
    SetWindowMode {
        mode: WindowMode,
        resolution: (u32, u32),
    },
    SetTitle {
        title: String,
    },
    SetScaleFactor {
        scale_factor: f64,
    },
    SetResolution {
        logical_resolution: (f32, f32),
        scale_factor: f64,
    },
    SetPresentMode {
        present_mode: PresentMode,
    },
    SetResizable {
        resizable: bool,
    },
    SetDecorations {
        decorations: bool,
    },
    SetCursorLockMode {
        locked: bool,
    },
    SetCursorIcon {
        icon: CursorIcon,
    },
    SetCursorVisibility {
        visible: bool,
    },
    SetCursorPosition {
        position: Vector2<f32>,
    },
    SetMaximized {
        maximized: bool,
    },
    SetMinimized {
        minimized: bool,
    },
    SetPosition {
        position: Vector2<i32>,
    },
    SetResizeConstraints {
        resize_constraints: WindowResizeConstraints,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    /// Creates a window that uses the given size
    Windowed,
    /// Creates a borderless window that uses the full size of the screen
    BorderlessFullscreen,
    /// Creates a fullscreen window that will render at desktop resolution. The app will use the closest supported size
    /// from the given size and scale it to fit the screen.
    SizedFullscreen,
    /// Creates a fullscreen window that uses the maximum supported size
    Fullscreen,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "vsync")]
pub enum PresentMode {
    /// The presentation engine does **not** wait for a vertical blanking period and
    /// the request is presented immediately. This is a low-latency presentation mode,
    /// but visible tearing may be observed. Will fallback to `Fifo` if unavailable on the
    /// selected platform and backend. Not optimal for mobile.
    Immediate = 0,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image, but frames may be submitted without delay. This is a low-latency
    /// presentation mode and visible tearing will **not** be observed. Will fallback to `Fifo`
    /// if unavailable on the selected platform and backend. Not optimal for mobile.
    Mailbox = 1,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image. The framerate will be capped at the display refresh rate,
    /// corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.
    Fifo = 2, // NOTE: The explicit ordinal values mirror wgpu and the vulkan spec.
}

/// The size limits on a window.
/// These values are measured in logical pixels, so the user's
/// scale factor does affect the size limits on the window.
/// Please note that if the window is resizable, then when the window is
/// maximized it may have a size outside of these limits. The functionality
/// required to disable maximizing is not yet exposed by winit.
#[derive(Debug, Clone, Copy)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 180.,
            min_height: 120.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}

impl WindowResizeConstraints {
    #[must_use]
    pub fn check_constraints(&self) -> Self {
        let WindowResizeConstraints {
            mut min_width,
            mut min_height,
            mut max_width,
            mut max_height,
        } = self;
        min_width = min_width.max(1.);
        min_height = min_height.max(1.);
        if max_width < min_width {
            // warn!(
            //     "The given maximum width {} is smaller than the minimum width {}",
            //     max_width, min_width
            // );
            max_width = min_width;
        }
        if max_height < min_height {
            // warn!(
            //     "The given maximum height {} is smaller than the minimum height {}",
            //     max_height, min_height
            // );
            max_height = min_height;
        }
        WindowResizeConstraints {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CursorIcon {
    Default,
    Crosshair,
    Hand,
    Arrow,
    Move,
    Text,
    Wait,
    Help,
    Progress,
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    Grab,
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

impl Into<winit::window::CursorIcon> for CursorIcon {
    fn into(self) -> winit::window::CursorIcon {
        match self {
            CursorIcon::Default => winit::window::CursorIcon::Default,
            CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
            CursorIcon::Hand => winit::window::CursorIcon::Hand,
            CursorIcon::Arrow => winit::window::CursorIcon::Arrow,
            CursorIcon::Move => winit::window::CursorIcon::Move,
            CursorIcon::Text => winit::window::CursorIcon::Text,
            CursorIcon::Wait => winit::window::CursorIcon::Wait,
            CursorIcon::Help => winit::window::CursorIcon::Help,
            CursorIcon::Progress => winit::window::CursorIcon::Progress,
            CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
            CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
            CursorIcon::Cell => winit::window::CursorIcon::Cell,
            CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
            CursorIcon::Alias => winit::window::CursorIcon::Alias,
            CursorIcon::Copy => winit::window::CursorIcon::Copy,
            CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
            CursorIcon::Grab => winit::window::CursorIcon::Grab,
            CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
            CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
            CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
            CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
            CursorIcon::EResize => winit::window::CursorIcon::EResize,
            CursorIcon::NResize => winit::window::CursorIcon::NResize,
            CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
            CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
            CursorIcon::SResize => winit::window::CursorIcon::SResize,
            CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
            CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
            CursorIcon::WResize => winit::window::CursorIcon::WResize,
            CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
            CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
            CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
            CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
            CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
            CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
        }
    }
}