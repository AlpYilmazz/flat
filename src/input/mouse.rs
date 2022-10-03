use super::{ButtonState, Input};
use bevy_ecs::{event::EventReader, system::ResMut};
use cgmath::Vector2;

/// Copied from bevy_input-0.8.1 - crate::mouse
#[derive(Debug, Clone)]
pub struct MouseButtonInput {
    /// The mouse button assigned to the event.
    pub button: MouseButton,
    /// The pressed state of the button.
    pub state: ButtonState,
}

/// Copied from bevy_input-0.8.1 - crate::mouse
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
// #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The right mouse button.
    Right,
    /// The middle mouse button.
    Middle,
    /// Another mouse button with the associated number.
    Other(u16),
}

#[derive(Debug, Clone)]
pub struct MouseMotion {
    /// The change in the position of the pointing device since the last event was sent.
    pub delta: Vector2<f32>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MouseScrollUnit {
    /// The line scroll unit.
    ///
    /// The delta of the associated [`MouseWheel`](crate::mouse::MouseWheel) event corresponds
    /// to the amount of lines or rows to scroll.
    Line,
    /// The pixel scroll unit.
    ///
    /// The delta of the associated [`MouseWheel`](crate::mouse::MouseWheel) event corresponds
    /// to the amount of pixels to scroll.
    Pixel,
}

#[derive(Debug, Clone)]
pub struct MouseWheel {
    /// The mouse scroll unit.
    pub unit: MouseScrollUnit,
    /// The horizontal scroll value.
    pub x: f32,
    /// The vertical scroll value.
    pub y: f32,
}

pub fn mouse_button_input_system(
    mut mouse_button_input: ResMut<Input<MouseButton>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    mouse_button_input.clear();
    for event in mouse_button_input_events.iter() {
        match event.state {
            ButtonState::Pressed => mouse_button_input.press(event.button),
            ButtonState::Released => mouse_button_input.release(event.button),
        }
    }
}

impl MouseButtonInput {
    pub fn from_with(button: winit::event::MouseButton, state: winit::event::ElementState) -> Self {
        Self {
            button: button.into(),
            state: state.into(),
        }
    }
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(val: winit::event::MouseButton) -> Self {
        match val {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}

impl From<winit::event::MouseScrollDelta> for MouseWheel {
    fn from(val: winit::event::MouseScrollDelta) -> Self {
        match val {
            winit::event::MouseScrollDelta::LineDelta(y, x) => MouseWheel {
                unit: MouseScrollUnit::Line,
                x,
                y,
            },
            winit::event::MouseScrollDelta::PixelDelta(pos) => MouseWheel {
                unit: MouseScrollUnit::Pixel,
                x: pos.x as f32,
                y: pos.y as f32,
            },
        }
    }
}

impl From<winit::dpi::PhysicalPosition<f64>> for MouseMotion {
    fn from(val: winit::dpi::PhysicalPosition<f64>) -> Self {
        MouseMotion {
            delta: cgmath::Vector2::new(val.x as f32, val.y as f32),
        }
    }
}

impl From<(f64, f64)> for MouseMotion {
    fn from(val: (f64, f64)) -> Self {
        MouseMotion {
            delta: cgmath::Vector2::new(val.0 as f32, val.1 as f32),
        }
    }
}
