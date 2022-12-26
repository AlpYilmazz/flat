use bevy::{
    asset::{load_internal_asset, HandleId},
    core::{Pod, Zeroable},
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::UniformComponentPlugin,
        render_asset::PrepareAssetLabel,
        render_phase::{
            sort_phase_system, AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{BindGroup, BufferVec, PipelineCache, SpecializedRenderPipelines},
        renderer::{RenderDevice, RenderQueue},
        view::{ViewUniformOffset, ViewUniforms, VisibleEntities},
        Extract, RenderApp, RenderStage,
    },
    utils::FloatOrd,
};

use crate::{
    core_2d::PrimitiveQuad,
    sprite::bind::{extract_sprites, prepare_model_uniforms},
};

use self::bind::{
    create_texture_bind_groups, ModelUniformOffset, ModelUniforms, SpritePipeline,
    SpritePipelineKey, TextureBindGroups,
};

pub mod bind;

const SPRITE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 45678909876445673);

pub struct FlatSpritePlugin;
impl Plugin for FlatSpritePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SPRITE_SHADER_HANDLE, "sprite.wgsl", Shader::from_wgsl);

        app.insert_resource(Msaa { samples: 1 });

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpritePipeline>()
                .init_resource::<SpecializedRenderPipelines<SpritePipeline>>()
                .init_resource::<ExtractedSprites>()
                .init_resource::<SpriteResources>()
                .init_resource::<ModelUniforms>()
                .init_resource::<TextureBindGroups>()
                .add_system_to_stage(RenderStage::Extract, extract_sprites)
                .add_system_to_stage(
                    RenderStage::Prepare,
                    create_texture_bind_groups.after(PrepareAssetLabel::AssetPrepare),
                )
                .add_system_to_stage(RenderStage::Prepare, prepare_model_uniforms)
                .add_system_to_stage(RenderStage::PhaseSort, sort_phase_system::<PrimitiveQuad>)
                .add_system_to_stage(RenderStage::Queue, queue_sprites)
                .add_render_command::<PrimitiveQuad, DrawSprite>();
        }
    }
}

#[derive(Bundle)]
pub struct SpriteBundle {
    pub sprite: Sprite,
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
}

#[derive(Component)]
pub struct Sprite {
    pub color: Color,
}

pub struct ExtractedSprite {
    pub entity: Entity,
    pub transform: GlobalTransform,
    pub image_handle: HandleId,
    pub color: Color,
}

#[derive(Resource, Default)]
pub struct ExtractedSprites {
    pub sprites: Vec<ExtractedSprite>,
}

#[derive(Component)]
pub struct GpuSprite {
    image_handle: HandleId,
    colored: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ColoredSpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Resource)]
pub struct SpriteResources {
    vertices: BufferVec<SpriteVertex>,
    colored_vertices: BufferVec<ColoredSpriteVertex>,
    model_bind_group: Option<BindGroup>,
    view_bind_group: Option<BindGroup>,
}

impl Default for SpriteResources {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(wgpu::BufferUsages::VERTEX),
            colored_vertices: BufferVec::new(wgpu::BufferUsages::VERTEX),
            model_bind_group: None,
            view_bind_group: None,
        }
    }
}

const QUAD_VERTEX_POSITIONS: [Vec3; 4] = [
    Vec3::new(-0.5, 0.5, 0.0),
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
];
const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
];
const QUAD_INDICES: [usize; 6] = [0, 1, 2, 2, 3, 0];

fn queue_sprites(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    draw_functions: Res<DrawFunctions<PrimitiveQuad>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    sprite_pipeline: Res<SpritePipeline>,
    mut specialized_sprite_pipelines: ResMut<SpecializedRenderPipelines<SpritePipeline>>,
    model_uniforms: Res<ModelUniforms>,
    view_uniforms: Res<ViewUniforms>,
    mut sprite_resources: ResMut<SpriteResources>,
    extracted_sprites: Res<ExtractedSprites>,
    mut render_phases: Query<(&mut RenderPhase<PrimitiveQuad>, &VisibleEntities)>,
) {
    let (Some(model_uniforms_binding), Some(view_uniforms_binding)) = 
        (model_uniforms.uniforms.binding(), view_uniforms.uniforms.binding()) else {
            return;
        };

    sprite_resources.model_bind_group =
        Some(render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_pipeline.model_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_uniforms_binding,
            }],
        }));
    sprite_resources.view_bind_group =
        Some(render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &sprite_pipeline.view_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_uniforms_binding,
            }],
        }));

    let draw_function = draw_functions.read().get_id::<DrawSprite>().unwrap();

    let pipeline = specialized_sprite_pipelines.specialize(
        &mut pipeline_cache,
        &sprite_pipeline,
        SpritePipelineKey::NONE,
    );
    let colored_pipeline = specialized_sprite_pipelines.specialize(
        &mut pipeline_cache,
        &sprite_pipeline,
        SpritePipelineKey::COLORED,
    );

    let mut colored_index = 0;
    let mut index = 0;

    let mut outer_cnt = 0;
    println!("-- Queue Sprites");
    for (mut render_phase, visible_entities) in render_phases.iter_mut() {
        println!("outer_cnt: {}, sprite_count: {}", outer_cnt, extracted_sprites.sprites.len());
        outer_cnt += 1;
        for sprite in &extracted_sprites.sprites {
            println!("-- for Phase, Sprite: {:?}", sprite.entity);
            // if !visible_entities.entities.contains(&sprite.entity) {
            //     continue;
            // }
            println!("-- visible");

            let current_entity = sprite.entity;
            let colored = sprite.color != Color::WHITE;

            commands.get_or_spawn(current_entity).insert(GpuSprite {
                image_handle: sprite.image_handle,
                colored,
            });

            let sort_key = FloatOrd(sprite.transform.translation().z);

            if colored {
                for i in QUAD_INDICES {
                    sprite_resources.colored_vertices.push(ColoredSpriteVertex {
                        position: QUAD_VERTEX_POSITIONS[i].into(),
                        uv: QUAD_UVS[i].into(),
                        color: sprite.color.as_linear_rgba_f32(),
                    });
                }
                let item_start = colored_index;
                colored_index += QUAD_INDICES.len() as u32;
                let item_end = colored_index;
                render_phase.items.push(PrimitiveQuad {
                    sort_key,
                    pipeline: colored_pipeline,
                    draw_function,
                    entity: current_entity,
                    item_range: item_start..item_end,
                });
            } else {
                for i in QUAD_INDICES {
                    sprite_resources.vertices.push(SpriteVertex {
                        position: QUAD_VERTEX_POSITIONS[i].into(),
                        uv: QUAD_UVS[i].into(),
                    });
                }
                let item_start = index;
                index += QUAD_INDICES.len() as u32;
                let item_end = index;
                render_phase.items.push(PrimitiveQuad {
                    sort_key,
                    pipeline,
                    draw_function,
                    entity: current_entity,
                    item_range: item_start..item_end,
                });
            }
        }

        sprite_resources
            .vertices
            .write_buffer(&render_device, &render_queue);
        sprite_resources
            .colored_vertices
            .write_buffer(&render_device, &render_queue);
    }
}

type DrawSprite = (
    SetItemPipeline,
    SetModelBindGroup<0>,
    SetViewBindGroup<1>,
    SetTextureBindGroup<2>,
    DrawSpriteSingle,
);

pub struct SetModelBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetModelBindGroup<I> {
    type Param = (SRes<SpriteResources>, SQuery<Read<ModelUniformOffset>>);

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (sprite_resources, offset_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_resources = sprite_resources.into_inner();
        pass.set_bind_group(
            I,
            sprite_resources.model_bind_group.as_ref().unwrap(),
            &[offset_query.get(item).unwrap().offset],
        );

        println!("1. model");

        RenderCommandResult::Success
    }
}

pub struct SetViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetViewBindGroup<I> {
    type Param = (SRes<SpriteResources>, SQuery<Read<ViewUniformOffset>>);

    fn render<'w>(
        view: Entity,
        _item: Entity,
        (sprite_resources, offset_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_resources = sprite_resources.into_inner();
        pass.set_bind_group(
            I,
            sprite_resources.view_bind_group.as_ref().unwrap(),
            &[offset_query.get(view).unwrap().offset],
        );

        println!("2. view");

        RenderCommandResult::Success
    }
}

pub struct SetTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTextureBindGroup<I> {
    type Param = (
        SRes<SpritePipeline>,
        SRes<TextureBindGroups>,
        SQuery<Read<GpuSprite>>,
    );

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (sprite_pipeline, texture_binds, gpu_sprite_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_pipeline = sprite_pipeline.into_inner();
        let texture_binds = texture_binds.into_inner();
        let texture_bind = texture_binds
            .0
            .get(&gpu_sprite_query.get(item).unwrap().image_handle)
            .unwrap_or_else(|| &sprite_pipeline.dummy_texture_bind_group);
        // .expect("Texture not ready");
        pass.set_bind_group(I, texture_bind, &[]);

        println!("3. texture");

        RenderCommandResult::Success
    }
}

pub struct DrawSpriteSingle;
impl RenderCommand<PrimitiveQuad> for DrawSpriteSingle {
    type Param = (SRes<SpriteResources>, SQuery<Read<GpuSprite>>);

    fn render<'w>(
        _view: Entity,
        item: &PrimitiveQuad,
        (sprite_resources, gpu_sprite_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_resources = sprite_resources.into_inner();
        let gpu_sprite = gpu_sprite_query.get(item.entity).unwrap();

        if gpu_sprite.colored {
            pass.set_vertex_buffer(
                0,
                sprite_resources
                    .colored_vertices
                    .buffer()
                    .unwrap()
                    .slice(..),
            );
        } else {
            pass.set_vertex_buffer(0, sprite_resources.vertices.buffer().unwrap().slice(..));
        }
        pass.draw(item.item_range.clone(), 0..1);

        println!("4. draw: item: {:?}", item);

        RenderCommandResult::Success
    }
}
