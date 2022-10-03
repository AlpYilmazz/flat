use std::{collections::HashSet, hash::Hash};

use bevy_app::Plugin;
use bevy_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};

use crate::CoreStage;

use self::mouse::MouseButton;
use self::{
    keyboard::{keyboard_input_system, KeyCode, KeyboardInput, ScanCode},
    mouse::{mouse_button_input_system, MouseButtonInput, MouseMotion, MouseWheel},
};

pub mod keyboard;
pub mod mouse;

#[derive(SystemLabel)]
pub struct InputSystem;

pub struct FlatInputPlugin;
impl Plugin for FlatInputPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_event::<ModifiersChanged>()
            .add_event::<KeyboardInput>()
            .init_resource::<Input<ScanCode>>()
            .init_resource::<Input<KeyCode>>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                keyboard_input_system.label(InputSystem),
            )
            .add_event::<MouseButtonInput>()
            .add_event::<MouseWheel>()
            .add_event::<MouseMotion>()
            .init_resource::<Input<MouseButton>>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                mouse_button_input_system.label(InputSystem),
            );
    }
}

#[derive(Debug, Clone)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl From<winit::event::ElementState> for ButtonState {
    fn from(val: winit::event::ElementState) -> Self {
        match val {
            winit::event::ElementState::Pressed => ButtonState::Pressed,
            winit::event::ElementState::Released => ButtonState::Released,
        }
    }
}

pub struct ModifiersChanged(pub ModifiersState);

bitflags::bitflags! {
    /// Represents the current state of the keyboard modifiers
    ///
    /// Each flag represents a modifier and is set if this modifier is active.
    #[derive(Default)]
    pub struct ModifiersState: u32 {
        // left and right modifiers are currently commented out, but we should be able to support
        // them in a future release
        /// The "shift" key.
        const SHIFT = 0b100;
        // const LSHIFT = 0b010;
        // const RSHIFT = 0b001;
        /// The "control" key.
        const CTRL = 0b100 << 3;
        // const LCTRL = 0b010 << 3;
        // const RCTRL = 0b001 << 3;
        /// The "alt" key.
        const ALT = 0b100 << 6;
        // const LALT = 0b010 << 6;
        // const RALT = 0b001 << 6;
        /// This is the "windows" key on PC and "command" key on Mac.
        const LOGO = 0b100 << 9;
        // const LLOGO = 0b010 << 9;
        // const RLOGO = 0b001 << 9;
    }
}

impl From<winit::event::ModifiersState> for ModifiersState {
    fn from(val: winit::event::ModifiersState) -> Self {
        let mut state = ModifiersState::empty();

        if val.shift() {
            state |= ModifiersState::SHIFT;
        }
        if val.ctrl() {
            state |= ModifiersState::CTRL;
        }
        if val.alt() {
            state |= ModifiersState::ALT;
        }
        if val.logo() {
            state |= ModifiersState::LOGO;
        }

        state
    }
}

/// Copied from bevy_input-0.8.1 - crate::input
#[derive(Debug, Clone)]
pub struct Input<T: Eq + Hash> {
    /// A collection of every button that is currently being pressed.
    pressed: HashSet<T>,
    /// A collection of every button that has just been pressed.
    just_pressed: HashSet<T>,
    /// A collection of every button that has just been released.
    just_released: HashSet<T>,
}

impl<T: Eq + Hash> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T> Input<T>
where
    T: Copy + Eq + Hash,
{
    /// Registers a press for the given `input`.
    pub fn press(&mut self, input: T) {
        // Returns `true` if the `input` wasn't pressed.
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    /// Returns `true` if the `input` has been pressed.
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has been pressed.
    pub fn any_pressed(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|it| self.pressed(it))
    }

    /// Registers a release for the given `input`.
    pub fn release(&mut self, input: T) {
        // Returns `true` if the `input` was pressed.
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    /// Registers a release for all currently pressed inputs.
    pub fn release_all(&mut self) {
        // Move all items from pressed into just_released
        self.just_released.extend(self.pressed.drain());
    }

    /// Returns `true` if the `input` has just been pressed.
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has just been pressed.
    pub fn any_just_pressed(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|it| self.just_pressed(it))
    }

    /// Clears the `just_pressed` state of the `input` and returns `true` if the `input` has just been pressed.
    ///
    /// Future calls to [`Input::just_pressed`] for the given input will return false until a new press event occurs.
    pub fn clear_just_pressed(&mut self, input: T) -> bool {
        self.just_pressed.remove(&input)
    }

    /// Returns `true` if the `input` has just been released.
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has just been released.
    pub fn any_just_released(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|it| self.just_released(it))
    }

    /// Clears the `just_released` state of the `input` and returns `true` if the `input` has just been released.
    ///
    /// Future calls to [`Input::just_released`] for the given input will return false until a new release event occurs.
    pub fn clear_just_released(&mut self, input: T) -> bool {
        self.just_released.remove(&input)
    }

    /// Clears the `pressed`, `just_pressed` and `just_released` data of the `input`.
    pub fn reset(&mut self, input: T) {
        self.pressed.remove(&input);
        self.just_pressed.remove(&input);
        self.just_released.remove(&input);
    }

    /// Clears the `pressed`, `just_pressed`, and `just_released` data for every input.
    ///
    /// See also [`Input::clear`] for simulating elapsed time steps.
    pub fn reset_all(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Clears the `just pressed` and `just released` data for every input.
    ///
    /// See also [`Input::reset_all`] for a full reset.
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// An iterator visiting every pressed input in arbitrary order.
    pub fn get_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.pressed.iter()
    }

    /// An iterator visiting every just pressed input in arbitrary order.
    pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_pressed.iter()
    }

    /// An iterator visiting every just released input in arbitrary order.
    pub fn get_just_released(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_released.iter()
    }
}
