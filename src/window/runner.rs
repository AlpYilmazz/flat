use bevy_app::AppExit;
use bevy_ecs::{
    event::ManualEventReader,
    prelude::Events,
    world::World,
};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

use crate::input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    ModifiersChanged, ModifiersState,
};

use super::{
    commands::{WindowCommands, WindowMode},
    events::{CreateWindow, CursorEntered, CursorLeft, FocusChanged, WindowCreated, RequestRedraw},
    util, Windows, WinitWindows,
};

pub fn execute_window_commands(world: &mut World) {
    let world = world.cell();
    let winit_windows = world.get_resource::<WinitWindows>().unwrap();
    let mut windows = world.get_resource_mut::<Windows>().unwrap();

    for (id, window) in windows.map.iter_mut() {
        for command in window.command_queue.drain(..) {
            let winit_window = winit_windows.map.get(id).unwrap();
            match command {
                WindowCommands::SetWindowMode {
                    mode,
                    resolution: (width, height),
                } => match mode {
                    WindowMode::Windowed => {
                        winit_window.set_fullscreen(None);
                    }
                    WindowMode::BorderlessFullscreen => {
                        winit_window
                            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    }
                    WindowMode::SizedFullscreen => {
                        winit_window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                            util::get_fitting_videomode(
                                winit_window.current_monitor().as_ref().unwrap(),
                                width,
                                height,
                            ),
                        )));
                    }
                    WindowMode::Fullscreen => {
                        winit_window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                            util::get_best_videomode(
                                winit_window.current_monitor().as_ref().unwrap(),
                            ),
                        )));
                    }
                },
                WindowCommands::SetTitle { title } => {
                    winit_window.set_title(&title);
                }
                WindowCommands::SetScaleFactor { .. } => {
                    // TODO
                }
                WindowCommands::SetResolution {
                    logical_resolution: (width, height),
                    scale_factor,
                } => {
                    winit_window.set_inner_size(
                        winit::dpi::LogicalSize::new(width, height)
                            .to_physical::<f64>(scale_factor),
                    );
                }
                WindowCommands::SetPresentMode { .. } => {}
                WindowCommands::SetResizable { resizable } => {
                    winit_window.set_resizable(resizable);
                }
                WindowCommands::SetDecorations { decorations } => {
                    winit_window.set_decorations(decorations);
                }
                WindowCommands::SetCursorLockMode { locked } => {
                    winit_window.set_cursor_grab(locked).unwrap_or_else(|_e| {});
                }
                WindowCommands::SetCursorIcon { icon } => {
                    winit_window.set_cursor_icon(icon.into());
                }
                WindowCommands::SetCursorVisibility { visible } => {
                    winit_window.set_cursor_visible(visible);
                }
                WindowCommands::SetCursorPosition { position } => {
                    // NOTE: What is this?
                    let inner_size = winit_window
                        .inner_size()
                        .to_logical::<f32>(winit_window.scale_factor());
                    winit_window
                        .set_cursor_position(winit::dpi::LogicalPosition::new(
                            position.x,
                            inner_size.height - position.y,
                        ))
                        .unwrap_or_else(|_e| {});
                }
                WindowCommands::SetMaximized { maximized } => {
                    winit_window.set_maximized(maximized);
                }
                WindowCommands::SetMinimized { minimized } => {
                    winit_window.set_minimized(minimized);
                }
                WindowCommands::SetPosition { position } => {
                    winit_window.set_outer_position(winit::dpi::PhysicalPosition {
                        x: position.x,
                        y: position.y,
                    });
                }
                WindowCommands::SetResizeConstraints { resize_constraints } => {
                    let constraints = resize_constraints.check_constraints();
                    let min_inner_size = winit::dpi::LogicalSize {
                        width: constraints.min_width,
                        height: constraints.min_height,
                    };
                    let max_inner_size = winit::dpi::LogicalSize {
                        width: constraints.max_width,
                        height: constraints.max_height,
                    };

                    winit_window.set_min_inner_size(Some(min_inner_size));
                    if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                        winit_window.set_max_inner_size(Some(max_inner_size));
                    }
                }
            }
        }
    }
}

pub fn winit_event_loop_runner(mut app: bevy_app::App) {
    let event_loop = app.world.remove_non_send_resource::<EventLoop<()>>().unwrap();
    app.insert_non_send_resource(event_loop.create_proxy());

    let mut redraw_event_reader = ManualEventReader::<RequestRedraw>::default();
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();

    event_loop.run(move |event0, event_loop_wt, control_flow| {
        match event0 {
            Event::NewEvents(_) => {}
            Event::WindowEvent {
                event,
                window_id: winit_window_id,
            } => match event {
                // WindowEvent::Resized(_) => {},
                // WindowEvent::Moved(_) => {},
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                // WindowEvent::Destroyed => {},
                // WindowEvent::DroppedFile(_) => {},
                // WindowEvent::HoveredFile(_) => {},
                // WindowEvent::HoveredFileCancelled => {},
                // WindowEvent::ReceivedCharacter(_) => {},
                WindowEvent::Focused(focused) => {
                    let world = app.world.cell();
                    let winit_windows = world.get_resource::<WinitWindows>().unwrap();
                    let window_id = winit_windows
                        .winit_to_lib
                        .get(&winit_window_id)
                        .unwrap()
                        .clone();
                    let mut events = world.get_resource_mut::<Events<FocusChanged>>().unwrap();
                    events.send(FocusChanged { window_id, focused });
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let world = app.world.cell();
                    let mut events = world.get_resource_mut::<Events<KeyboardInput>>().unwrap();
                    events.send(KeyboardInput::from(input));
                }
                WindowEvent::ModifiersChanged(state) => {
                    let world = app.world.cell();
                    let mut events = world.get_resource_mut::<Events<ModifiersChanged>>().unwrap();
                    events.send(ModifiersChanged(ModifiersState::from(state)));
                }
                // WindowEvent::CursorMoved {
                //     device_id,
                //     position,
                //     modifiers,
                // } => {},
                WindowEvent::CursorEntered { .. } => {
                    let world = app.world.cell();
                    let winit_windows = world.get_resource::<WinitWindows>().unwrap();
                    let window_id = winit_windows
                        .winit_to_lib
                        .get(&winit_window_id)
                        .unwrap()
                        .clone();
                    let mut events = world.get_resource_mut::<Events<CursorEntered>>().unwrap();
                    events.send(CursorEntered { window_id });
                }
                WindowEvent::CursorLeft { .. } => {
                    let world = app.world.cell();
                    let winit_windows = world.get_resource::<WinitWindows>().unwrap();
                    let window_id = winit_windows
                        .winit_to_lib
                        .get(&winit_window_id)
                        .unwrap()
                        .clone();
                    let mut events = world.get_resource_mut::<Events<CursorLeft>>().unwrap();
                    events.send(CursorLeft { window_id });
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let world = app.world.cell();
                    let mut events = world.get_resource_mut::<Events<MouseWheel>>().unwrap();
                    events.send(MouseWheel::from(delta));
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let world = app.world.cell();
                    let mut events = world.get_resource_mut::<Events<MouseButtonInput>>().unwrap();
                    events.send(MouseButtonInput::from_with(button, state));
                }
                // WindowEvent::TouchpadPressure {
                //     device_id,
                //     pressure,
                //     stage,
                // } => {},
                // WindowEvent::AxisMotion {
                //     device_id,
                //     axis,
                //     value,
                // } => {},
                // WindowEvent::Touch(_) => {},
                // WindowEvent::ScaleFactorChanged {
                //     scale_factor,
                //     new_inner_size,
                // } => {},
                // WindowEvent::ThemeChanged(_) => {},
                _ => (),
            },
            Event::DeviceEvent { device_id: _, event } => {
                match event {
                    DeviceEvent::Added => {}
                    DeviceEvent::Removed => {}
                    DeviceEvent::MouseMotion { delta } => {
                        let world = app.world.cell();
                        let mut events = world.get_resource_mut::<Events<MouseMotion>>().unwrap();
                        events.send(MouseMotion::from(delta));
                    }
                    // DeviceEvent::MouseWheel { delta } => {},
                    // DeviceEvent::Motion { axis, value } => {},
                    // DeviceEvent::Button { button, state } => {},
                    // DeviceEvent::Key(_) => {},
                    // DeviceEvent::Text { codepoint } => {},
                    _ => (),
                }
            }
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::Resumed => {}
            Event::MainEventsCleared => {
                handle_create_window(&mut app.world, event_loop_wt);
                // NOTE: this is why you cannot borrow app at the top
                app.update();
            }
            Event::RedrawRequested(_) => {}
            Event::RedrawEventsCleared => {
                if let Some(app_redraw_events) = app.world.get_resource::<Events<RequestRedraw>>() {
                    if redraw_event_reader.iter(app_redraw_events).last().is_some() {
                        *control_flow = ControlFlow::Poll;
                    }
                }
                if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
                    if app_exit_event_reader.iter(app_exit_events).last().is_some() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            Event::LoopDestroyed => {}
            // Event::RedrawRequested(window_id) => {
            //     app.update();
            // }
            // Event::MainEventsCleared => {
            //     // RedrawRequested will only trigger once, unless we manually
            //     // request it.
            //     window.request_redraw();
            // }
            // _ => (),
        }
    });
}

pub fn handle_create_window(
    world: &mut World,
    event_loop: &EventLoopWindowTarget<()>,
    // reader: &mut ManualEventReader<CreateWindow>,
) {
    let world = world.cell();
    let mut winit_windows = world.get_resource_mut::<WinitWindows>().unwrap();
    let mut windows = world.get_resource_mut::<Windows>().unwrap();
    let mut create_events = world.get_resource_mut::<Events<CreateWindow>>().unwrap();
    let mut window_created_events = world.get_resource_mut::<Events<WindowCreated>>().unwrap();

    for event in create_events.drain() {
        let window = winit_windows.create_window(event_loop, event.id, event.desc);
        windows.add(window);
        window_created_events.send(WindowCreated {
            id: event.id,
        });
    }
}
