// TODO: investigate instancing as an option - since we already have all the
// vertex data it might be feasible to to do without any extra buffers...
// <https://bevyengine.org/examples/shader/shader-instancing/>

use std::mem;
use std::num::NonZeroU64;

use bevy::log;
use bevy::pbr::{
    extract_materials, extract_meshes, prepare_materials, ExtendedMaterial, ExtractedMaterials,
    MaterialPipeline, MaterialPipelineKey, RenderMaterials,
};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_asset::{prepare_assets, PrepareAssetSet, RenderAssets};
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferInitDescriptor, BufferUsages, PreparedBindGroup, RenderPipelineDescriptor, ShaderRef,
    ShaderStages, SpecializedMeshPipelineError, UnpreparedBindGroup,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::FallbackImage;
use bevy::render::{RenderApp, RenderSet};

mod pipeline;

pub struct BubblesMaterialPlugin;

impl Plugin for BubblesMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<BubblesMaterial>::default());

        app.sub_app_mut(RenderApp).add_system(
            extract_vertex_buffers
                .in_set(RenderSet::Prepare)
                .after(prepare_assets::<Mesh>)
                .after(prepare_materials::<BubblesMaterial>),
        );
    }
}

pub fn from_standard_material(standard: StandardMaterial) -> BubblesMaterial {
    BubblesMaterial {
        standard: StandardMaterial {
            cull_mode: None,
            ..standard
        },
        extended: default(),
    }
}

pub type BubblesMaterial = ExtendedMaterial<Bubbles>;

#[derive(TypeUuid, Debug, Clone)]
#[uuid = "68c25f8b-b16a-4630-aa6c-e0399e71fbd6"]
pub struct Bubbles {
    /// How big the bubbles should be
    pub bubble_radius: f32,

    vertex_buffer: Option<Buffer>,
}

impl Default for Bubbles {
    fn default() -> Self {
        Self {
            bubble_radius: 1.0,
            vertex_buffer: None,
        }
    }
}

impl AsBindGroup for Bubbles {
    type Data = ();

    fn label() -> Option<&'static str> {
        Some("bubbles_vertex_buffer")
    }

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        _images: &RenderAssets<Image>,
        _fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let Some(vertex_buffer) = &self.vertex_buffer
        else { return Err(AsBindGroupError::RetryNextUpdate) };

        let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Self::label(),
            contents: bytemuck::cast_slice(&[self.bubble_radius]),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Self::label(),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 100,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 101,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: vertex_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Ok(PreparedBindGroup {
            bindings: Vec::new(),
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _layout: &BindGroupLayout,
        _render_device: &RenderDevice,
        _images: &RenderAssets<Image>,
        _fallback_image: &FallbackImage,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        unimplemented!("this shouldn't be called since `as_bind_group` is implemented")
    }

    fn bind_group_layout_entries(_render_device: &RenderDevice) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        vec![
            BindGroupLayoutEntry {
                binding: 100,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(mem::size_of::<f32>() as u64),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 101,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
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
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(label) = &mut descriptor.label {
            *label = format!("bubbles_{label}").into();
        }

        Ok(())
    }
}

fn extract_vertex_buffers(
    mesh_materials: Query<(Entity, &Handle<Mesh>, &Handle<BubblesMaterial>)>,
    meshes: Res<RenderAssets<Mesh>>,
    mut render_materials: ResMut<RenderMaterials<BubblesMaterial>>,
) {
    for (entity, mesh_handle, material_handle) in &mesh_materials {
        let Some(material) = render_materials.get(mesh_handle)
        else { continue };

        for (extracted_handle, extracted) in &mut materials {
            if extracted_handle == material_handle {
                let Some(mesh) = meshes.get(mesh_handle) else {
                    log::error!("failed to get mesh {mesh_handle:?}");
                    continue;
                };

                extracted.extended.vertex_buffer = Some(mesh.vertex_buffer);
            }
        }
    }
}
