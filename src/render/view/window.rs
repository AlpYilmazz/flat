use bevy::{
    prelude::{Deref, DerefMut, Plugin, Res, ResMut, Resource},
    utils::HashMap,
    window::{RawHandleWrapper, WindowId, Windows},
};

use crate::render::{
    camera,
    texture::{self, DepthTextures},
    RenderAdapter, RenderDevice, RenderInstance, RenderStage,
};

pub struct FlatViewPlugin;
impl Plugin for FlatViewPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<WindowSurfaces>()
            .init_resource::<PreparedWindows>()
            .add_system_to_stage(RenderStage::Prepare, prepare_windows)
            .add_system_to_stage(RenderStage::Create, configure_surfaces);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct WindowSurfaces(pub HashMap<WindowId, (wgpu::Surface, wgpu::TextureFormat)>);

pub struct SurfaceTextureData {
    // Drop view first
    pub view: wgpu::TextureView,
    pub texture: wgpu::SurfaceTexture,
}

pub struct PreparedWindow {
    pub id: WindowId,
    pub raw_handle: Option<RawHandleWrapper>,
    pub physical_width: u32,
    pub physical_height: u32,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
    pub surface_texture: Option<SurfaceTextureData>,
    pub surface_texture_format: Option<wgpu::TextureFormat>,
    pub size_changed: bool,
    pub present_mode_changed: bool,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PreparedWindows(pub HashMap<WindowId, PreparedWindow>);

pub fn prepare_windows(windows: Res<Windows>, mut prepared_windows: ResMut<PreparedWindows>) {
    for window in windows.iter() {
        let (new_width, new_height) = (window.physical_width(), window.physical_height());
        let new_present_mode = match window.present_mode() {
            bevy::window::PresentMode::AutoVsync => wgpu::PresentMode::AutoVsync,
            bevy::window::PresentMode::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
            bevy::window::PresentMode::Immediate => wgpu::PresentMode::Immediate,
            bevy::window::PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
            bevy::window::PresentMode::Fifo => wgpu::PresentMode::Fifo,
        };
        let alpha_mode = match window.alpha_mode() {
            bevy::window::CompositeAlphaMode::Auto => wgpu::CompositeAlphaMode::Auto,
            bevy::window::CompositeAlphaMode::Opaque => wgpu::CompositeAlphaMode::Opaque,
            bevy::window::CompositeAlphaMode::PreMultiplied => {
                wgpu::CompositeAlphaMode::PreMultiplied
            }
            bevy::window::CompositeAlphaMode::PostMultiplied => {
                wgpu::CompositeAlphaMode::PostMultiplied
            }
            bevy::window::CompositeAlphaMode::Inherit => wgpu::CompositeAlphaMode::Inherit,
        };

        let prep_window = prepared_windows
            .entry(window.id())
            .or_insert_with(|| PreparedWindow {
                id: window.id(),
                raw_handle: window.raw_handle(),
                physical_width: new_width,
                physical_height: new_height,
                present_mode: new_present_mode,
                alpha_mode: alpha_mode,
                surface_texture: None,
                surface_texture_format: None,
                size_changed: false,
                present_mode_changed: false,
            });

        prep_window.surface_texture = None;
        prep_window.size_changed =
            new_width != prep_window.physical_width || new_height != prep_window.physical_height;
        prep_window.present_mode_changed = new_present_mode != prep_window.present_mode;

        prep_window.physical_width = new_width;
        prep_window.physical_height = new_height;
        prep_window.present_mode = new_present_mode;
    }
}

pub fn configure_surfaces(
    render_instance: Res<RenderInstance>,
    render_adapter: Res<RenderAdapter>,
    render_device: Res<RenderDevice>,
    mut windows: ResMut<PreparedWindows>,
    mut surfaces: ResMut<WindowSurfaces>,
    mut depth_textures: ResMut<DepthTextures>,
) {
    for window in windows.values_mut() {
        let is_new_surface = !surfaces.contains_key(&window.id);
        let (surface, format) = surfaces.entry(window.id).or_insert_with(|| unsafe {
            let surface =
                render_instance.create_surface(&window.raw_handle.as_ref().unwrap().get_handle());
            let format = surface
                .get_supported_formats(&render_adapter)
                .get(0)
                .cloned()
                .expect("No supported formats");
            (surface, format)
        });

        if window.physical_width == 0 || window.physical_height == 0 {
            continue;
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: format.clone(),
            width: window.physical_width,
            height: window.physical_height,
            present_mode: window.present_mode,
            alpha_mode: window.alpha_mode,
        };

        if is_new_surface || window.size_changed || window.present_mode_changed {
            surface.configure(render_device.inner(), &config);
            let surface_texture = surface
                .get_current_texture()
                .expect("Could not get surface texture");
            let surface_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            window.surface_texture = Some(SurfaceTextureData {
                view: surface_view,
                texture: surface_texture,
            });

            // TODO: support RenderTarget::Image
            // NOTE: creates depth texture for all windows
            match depth_textures.get_mut(&camera::component::RenderTarget::Window(window.id)) {
                Some(dt) => {
                    *dt = texture::DepthTexture::create(&render_device, &config);
                }
                None => {
                    depth_textures.insert(
                        camera::component::RenderTarget::Window(window.id),
                        texture::DepthTexture::create(&render_device, &config),
                    );
                }
            }
        } else {
            match surface.get_current_texture() {
                Ok(st) => {
                    window.surface_texture = Some(SurfaceTextureData {
                        view: st
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                        texture: st,
                    });
                }
                Err(wgpu::SurfaceError::Outdated) => {
                    surface.configure(render_device.inner(), &config);
                    let surface_texture = surface
                        .get_current_texture()
                        .expect("Could not get surface texture");
                    let surface_view = surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    window.surface_texture = Some(SurfaceTextureData {
                        view: surface_view,
                        texture: surface_texture,
                    });

                    // TODO: support RenderTarget::Image
                    if let Some(dt) =
                        depth_textures.get_mut(&camera::component::RenderTarget::Window(window.id))
                    {
                        *dt = texture::DepthTexture::create(&render_device, &config);
                    }
                }
                Err(_) => {
                    panic!("Could not get surface texture");
                }
            }
        }
    }
}
