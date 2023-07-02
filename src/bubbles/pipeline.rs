//! Sketch of what a custom pipeline might look like, to plumb the mesh vertex buffer
//! into the material shader as a buffer uniform.

use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::{lifetimeless::*, SystemParamItem};
use bevy::pbr::{
    MaterialPipeline, MaterialPipelineKey, MeshPipeline, MeshPipelineKey, MeshUniform,
    RenderMaterials, SetMeshBindGroup, SetMeshViewBindGroup,
};
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::mesh::{GpuBufferInfo, MeshVertexBufferLayout};
use bevy::render::render_asset::*;
use bevy::render::render_phase::*;
use bevy::render::render_resource::{
    AsBindGroup, BindGroupDescriptor, BindGroupEntry, OwnedBindingResource, PipelineCache,
    RenderPipelineDescriptor, SpecializedMeshPipeline, SpecializedMeshPipelineError,
    SpecializedMeshPipelines,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ExtractedView;
use bevy::{log, prelude::*};

use super::BubblesMaterial;

pub type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    // SetMaterialBindGroup<BubblesMaterial, 1>, // skipped because we set the bind group in `Draw`
    SetMeshBindGroup<2>, // we pass our own quad mesh data instead
    DrawBubblesMaterial,
);

pub fn queue_draw_bubbles(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    material_pipeline: Res<MaterialPipeline<BubblesMaterial>>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<MaterialPipeline<BubblesMaterial>>>,
    pipeline_cache: Res<PipelineCache>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_materials: Res<RenderMaterials<BubblesMaterial>>,
    material_meshes: Query<(
        Entity,
        &Handle<BubblesMaterial>,
        &MeshUniform,
        &Handle<Mesh>,
    )>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (entity, material_handle, mesh_uniform, mesh_handle) in &material_meshes {
            if let (Some(mesh), Some(material)) = (
                render_meshes.get(mesh_handle),
                render_materials.get(material_handle),
            ) {
                let key = MaterialPipelineKey {
                    mesh_key: view_key
                        | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology),
                    bind_group_data: material.key.clone(),
                };

                let pipeline = pipelines
                    .specialize(&pipeline_cache, &material_pipeline, key, &mesh.layout)
                    .unwrap();

                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

pub struct DrawBubblesMaterial;

impl<P: PhaseItem> RenderCommand<P> for DrawBubblesMaterial {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SRes<RenderMaterials<BubblesMaterial>>,
    );

    type ViewWorldQuery = ();

    type ItemWorldQuery = (
        Read<Handle<Mesh>>,
        Read<Handle<BubblesMaterial>>,
        Read<DynamicUniformIndex<MeshUniform>>,
    );

    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, material_handle, mesh_uniform): ROQueryItem<'_, Self::ItemWorldQuery>,
        (meshes, prepared_materials): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (Some(prepared_material), Some(mesh)) = (
            prepared_materials.into_inner().get(material_handle),
            meshes.into_inner().get(mesh_handle),
        ) else { return RenderCommandResult::Failure };

        let Some((_ , OwnedBindingResource::Buffer(buf))) = prepared_material
            .bindings
            .iter()
            .find(|(binding, _)| *binding == 102)
        else { return RenderCommandResult::Failure };

        // pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(0, buf.slice(..));

        // StandardMaterial normally sets bind group 1
        pass.set_bind_group(1, &prepared_material.bind_group, &[]);

        match &mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                count,
                index_format,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..6, 0, 0..*count);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..6, 0..*vertex_count);
            }
        }

        RenderCommandResult::Success
    }
}
