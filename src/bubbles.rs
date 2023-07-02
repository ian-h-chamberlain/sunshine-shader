use std::mem;
use std::sync::Once;

use bevy::core::{Pod, Zeroable};
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::log;
use bevy::pbr::{
    extract_materials, prepare_materials, queue_material_meshes, ExtendedMaterial,
    ExtractedMaterials, MaterialPipeline, MaterialPipelineKey, PrepassPlugin, RenderMaterials,
};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_asset::{prepare_assets, PrepareAssetSet, RenderAssets};
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages,
    OwnedBindingResource, RawVertexBufferLayout, RenderPipelineDescriptor, ShaderRef, ShaderType,
    SpecializedMeshPipelineError, SpecializedMeshPipelines, UnpreparedBindGroup, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::FallbackImage;
use bevy::render::{RenderApp, RenderSet};

use self::pipeline::{queue_draw_bubbles, DrawCustom};

mod pipeline;

pub struct BubblesMaterialPlugin;

impl Plugin for BubblesMaterialPlugin {
    fn build(&self, app: &mut App) {
        // mostly copied from MaterialPlugin<M>:

        app.add_asset::<BubblesMaterial>()
            .add_plugin(ExtractComponentPlugin::<Handle<BubblesMaterial>>::default());

        app.sub_app_mut(RenderApp)
            .init_resource::<MaterialPipeline<BubblesMaterial>>()
            .init_resource::<ExtractedMaterials<BubblesMaterial>>()
            .init_resource::<RenderMaterials<BubblesMaterial>>()
            .init_resource::<SpecializedMeshPipelines<MaterialPipeline<BubblesMaterial>>>()
            .add_system_to_schedule(ExtractSchedule, extract_materials::<BubblesMaterial>)
            .add_system(
                prepare_materials::<BubblesMaterial>
                    .in_set(RenderSet::Prepare)
                    .after(PrepareAssetSet::PreAssetPrepare),
            )
            .add_render_command::<Transparent3d, DrawCustom>()
            .add_system(
                prepare_bubble_material
                    .in_set(RenderSet::Prepare)
                    .after(prepare_materials::<BubblesMaterial>),
            )
            .add_system(
                queue_draw_bubbles
                    .in_set(RenderSet::Queue)
                    .after(queue_material_meshes::<BubblesMaterial>),
            );
    }
}

pub type BubblesMaterial = ExtendedMaterial<Bubbles>;

pub fn material_from_standard(standard: StandardMaterial) -> BubblesMaterial {
    BubblesMaterial {
        standard: StandardMaterial {
            cull_mode: None,
            ..standard
        },
        extended: default(),
    }
}

fn prepare_bubble_material(
    mut prepared_materials: ResMut<RenderMaterials<BubblesMaterial>>,
    meshes: Res<RenderAssets<Mesh>>,
    render_device: Res<RenderDevice>,
    query: Query<(&Handle<Mesh>, &Handle<BubblesMaterial>)>,
) {
    for (mesh_handle, material_handle) in &query {
        let Some(prepared_material) = prepared_materials.get_mut(material_handle)
        else {
            log::error!("no mat found for {material_handle:?}");
            continue;
        };

        let Some(mesh) = meshes.get(mesh_handle)
        else {
            log::error!("no mesh found for {mesh_handle:?}");
            continue;
        };

        static LOG_IT: Once = Once::new();
        LOG_IT.call_once(|| {
            log::debug!("layout is {:#?}", mesh.layout);
        });

        for binding in &mut prepared_material.bindings {
            if let (101, OwnedBindingResource::Buffer(buf)) = binding {
                // swap out our dummy buffer for the real mesh buffer
                let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("bubble vertex buf"),
                    // TODO: see `impl RenderAsset for Mesh`, perhaps we'd need
                    // a separate RenderAsset impl but damn that sucks if so...
                    contents: todo!("how do get data??"),
                    usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
                });
                *buf = buffer;
            }
        }

        // recreate the bind group after we update the bindings, maybe this will
        // fix the vertex buffer not working right?
        let entries = prepared_material
            .bindings
            .iter()
            .map(|(index, binding)| BindGroupEntry {
                binding: *index,
                resource: binding.get_binding(),
            })
            .collect::<Vec<_>>();

        let layout = BubblesMaterial::bind_group_layout(&render_device);

        prepared_material.bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("bubbles bind group"),
            layout: &layout,
            entries: &entries,
        });

        // HACK: smuggling this along as an "owned binding resource" is probably
        // not really correct, and we should stick it in a dedicated mesh resource
        // or something.
        //
        // Also, we could probably use the AABB of the mesh (maybe with some margins,
        // relative to the bubble radius) to avoid rendering lots of fragments
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bubbles render quad"),
            contents: bytemuck::cast_slice(&[
                // z = 0, so that the quad renders in front of anything else
                Vec3::new(0.4, 0.8, 0.0),   // top-right
                Vec3::new(-0.4, 0.8, 0.0),  // top-left
                Vec3::new(-0.4, -0.8, 0.0), // bottom-left
                Vec3::new(-0.4, -0.8, 0.0), // again
                Vec3::new(0.4, -0.8, 0.0),  // bottom-right
                Vec3::new(0.4, 0.8, 0.0),   // top-right
            ]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        prepared_material
            .bindings
            .push((102, OwnedBindingResource::Buffer(buffer)));
    }
}

pub const MESH_VERTEX_BUFFER_BINDING: u32 = 101;

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "68c25f8b-b16a-4630-aa6c-e0399e71fbd6"]
pub struct Bubbles {
    /// How big the bubbles should be
    #[uniform(100)]
    pub bubble_radius: f32,

    /// A binding to reuse the vertex buffer as storage.
    /// This binding will describe
    #[storage(101, read_only)]
    pub mesh_vertex_buffer: Vec<Vertex>,
}

/// A helper struct to represent the type of elements in the mesh vertex buffer,
/// to make it easier to derive [`AsBindGroup`] for [`Bubbles`].
#[derive(Debug, Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

impl Default for Bubbles {
    fn default() -> Self {
        Self {
            bubble_radius: 1.0,
            mesh_vertex_buffer: Vec::new(),
        }
    }
}

impl Material for Bubbles {
    fn vertex_shader() -> ShaderRef {
        "shaders/bubbles.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/bubbles.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(label) = &mut descriptor.label {
            *label = format!("bubbles_{label}").into();
        }

        // add another vertex buffer for the fullscreen quad so we render every fragment
        descriptor.vertex.buffers = vec![VertexBufferLayout {
            step_mode: VertexStepMode::Vertex,
            array_stride: mem::size_of::<Vec3>() as u64,
            attributes: vec![VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                // TODO maybe we can actually just use normal 0 here???
                // okay yes you can but it has to match the vertex shader input,
                // and only works if replacing `descriptor.vertex.buffers` rather
                // than pushing to it
                shader_location: 0,
            }],
        }];

        log::debug!(
            "added vertex buffer layout, we now have {} bufs (should be 2)",
            descriptor.vertex.buffers.len()
        );

        Ok(())
    }
}
