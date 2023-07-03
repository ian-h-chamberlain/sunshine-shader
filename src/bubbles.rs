use std::mem;
use std::sync::Once;

use bevy::core::{Pod, Zeroable};
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::log;
use bevy::pbr::{
    extract_materials, prepare_materials, queue_material_meshes, ExtendedMaterial,
    ExtractedMaterials, MaterialPipeline, MaterialPipelineKey, PrepassPlugin, RenderMaterials,
};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::{GpuBufferInfo, GpuMesh, MeshVertexBufferLayout, VertexAttributeValues};
use bevy::render::render_asset::{
    prepare_assets, ExtractedAssets, PrepareAssetError, PrepareAssetSet, RenderAsset, RenderAssets,
};
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferInitDescriptor, BufferUsages, OwnedBindingResource,
    RawVertexBufferLayout, RenderPipelineDescriptor, ShaderRef, ShaderType,
    SpecializedMeshPipelineError, SpecializedMeshPipelines, UnpreparedBindGroup, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::FallbackImage;
use bevy::render::{Extract, RenderApp, RenderSet};
use bevy::utils::{HashMap, HashSet};

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
            .init_resource::<ExtractedMeshes>()
            .init_resource::<SpecializedMeshPipelines<MaterialPipeline<BubblesMaterial>>>()
            .add_system_to_schedule(ExtractSchedule, extract_materials::<BubblesMaterial>)
            .add_system_to_schedule(ExtractSchedule, extract_meshes)
            .add_system(
                prepare_materials::<BubblesMaterial>
                    .in_set(RenderSet::Prepare)
                    .after(PrepareAssetSet::PreAssetPrepare),
            )
            .add_render_command::<Transparent3d, DrawCustom>()
            .add_system(
                prepare_bubble_material
                    .in_set(RenderSet::Prepare)
                    .after(prepare_materials::<BubblesMaterial>)
                    .after(prepare_assets::<Mesh>),
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
            alpha_mode: AlphaMode::Blend,
            ..standard
        },
        extended: default(),
    }
}

#[derive(Resource, Debug, Default)]
struct ExtractedMeshes {
    extracted: HashMap<Handle<Mesh>, Mesh>,
    removed: HashSet<Handle<Mesh>>,
}

// This isn't quite how normal extraction works, but I think it's okay to just keep
// all the meshes around forever, updating instead of processing changes each frame
fn extract_meshes(
    mut events: Extract<EventReader<AssetEvent<Mesh>>>,
    assets: Extract<Res<Assets<Mesh>>>,
    mut extracted_meshes: ResMut<ExtractedMeshes>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let mut extracted = HashMap::new();
    for handle in changed_assets.drain() {
        if let Some(asset) = assets.get(&handle) {
            extracted.insert(handle, asset.extract_asset());
        }
    }

    extracted_meshes.extracted.extend(extracted);
    extracted_meshes.removed.extend(removed);
}

fn prepare_bubble_material(
    mut prepared_materials: ResMut<RenderMaterials<BubblesMaterial>>,
    meshes: Res<ExtractedMeshes>,
    render_device: Res<RenderDevice>,
    query: Query<(&Handle<Mesh>, &Handle<BubblesMaterial>)>,
) {
    for (mesh_handle, material_handle) in &query {
        let Some(prepared_material) = prepared_materials.get_mut(material_handle)
        else {
            log::error!("no mat found for {material_handle:?}");
            continue;
        };

        let Some(mesh) = meshes.extracted.get(mesh_handle)
        else {
            log::error!("no mesh found for {mesh_handle:?}: actual is {:?}", &meshes);
            continue;
        };
        let vertex_buffer_data = mesh.get_vertex_buffer_data();

        let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bubble vertex buf"),
            contents: &vertex_buffer_data,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
        });

        let mut layout_entries = BubblesMaterial::bind_group_layout_entries(&render_device);

        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            for layout_entry in &mut layout_entries {
                if layout_entry.binding == 101 {
                    log::debug!("vertex storage layout is {layout_entry:#?}");
                    log::debug!("vertex has size {}", mem::size_of::<Vertex>());
                }
            }

            log::debug!("prepared material layout: {layout_entries:#?}");
        });

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bubbles bind group layout"),
            entries: &layout_entries,
        });

        let entries = prepared_material
            .bindings
            .iter()
            .map(|(index, binding)| {
                let resource = if *index == 101 {
                    vertex_buffer.as_entire_binding()
                } else {
                    binding.get_binding()
                };

                BindGroupEntry {
                    binding: *index,
                    resource,
                }
            })
            .collect::<Vec<_>>();

        prepared_material.bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("bubbles bind group"),
            layout: &layout,
            entries: &entries,
        });

        // HACK: smuggling this along as an "owned binding resource" is probably
        // not really correct, and we should stick it in a dedicated mesh resource
        // or something.
        //
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bubbles render quad"),
            contents: bytemuck::cast_slice(geom::QUAD_MESH),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        prepared_material
            .bindings
            .push((102, OwnedBindingResource::Buffer(buffer)));
    }
}

mod geom {
    use bevy::prelude::*;

    // z = 0, so that the quad renders in front of anything else. Ideally it would
    // be nice to have this cover the whole viewport, but for now that's too expensive
    const TOP_LEFT: Vec3 = Vec3::new(-0.4, 0.8, 0.0);
    const TOP_RIGHT: Vec3 = Vec3::new(0.4, 0.8, 0.0);
    const BOT_LEFT: Vec3 = Vec3::new(-0.4, -0.8, 0.0);
    const BOT_RIGHT: Vec3 = Vec3::new(0.4, -0.8, 0.0);

    pub static QUAD_MESH: &[Vec3] = &[
        TOP_LEFT, BOT_LEFT, TOP_RIGHT, // upper-left half of the quad
        TOP_RIGHT, BOT_LEFT, BOT_RIGHT, // bottom-right half of the quad
    ];
}

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
                shader_location: 0,
            }],
        }];

        log::debug!(
            "updated vertex buffer layout: {:#?}",
            descriptor.vertex.buffers,
        );

        Ok(())
    }
}
