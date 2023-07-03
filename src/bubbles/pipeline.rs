//! Sketch of what a custom pipeline might look like, to plumb the mesh vertex buffer
//! into the material shader as a buffer uniform.

use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::pbr::{
    MaterialPipeline, MaterialPipelineKey, MeshPipelineKey, MeshUniform, RenderMaterials,
    SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup,
};
use bevy::prelude::*;
use bevy::render::mesh::GpuBufferInfo;
use bevy::render::render_asset::*;
use bevy::render::render_phase::*;
use bevy::render::render_resource::{
    OwnedBindingResource, PipelineCache, SpecializedMeshPipelines,
};
use bevy::render::view::ExtractedView;
use inline_tweak::tweak;

use super::BubblesMaterial;

pub type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMaterialBindGroup<BubblesMaterial, 1>, // skipped because we set the bind group in `Draw`
    SetMeshBindGroup<2>,
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

    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<Handle<BubblesMaterial>>);

    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, material_handle): ROQueryItem<'_, Self::ItemWorldQuery>,
        (meshes, prepared_materials): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (Some(prepared_material), Some(mesh)) = (
            prepared_materials.into_inner().get(material_handle),
            meshes.into_inner().get(mesh_handle),
        ) else { return RenderCommandResult::Failure };

        let Some((_ , OwnedBindingResource::Buffer(quad_vertex_buffer))) = prepared_material
            .bindings
            .iter()
            .find(|(binding, _)| *binding == 102)
        else { return RenderCommandResult::Failure };

        pass.set_vertex_buffer(0, quad_vertex_buffer.slice(..));

        let instance_count = match &mesh.buffer_info {
            GpuBufferInfo::Indexed { count, .. } => *count,
            GpuBufferInfo::NonIndexed { vertex_count } => *vertex_count,
        };

        // we know the quad buffer is non-indexed with fixed number of verts,
        // draw it directly. clamp the instance count for performance, but
        // ideally we really ought to be skipping a bunch of tris to trim this
        // down *before* sending to the GPU. Maybe extraction could do that
        pass.draw(0..6, 0..instance_count.min(tweak!(200)));

        RenderCommandResult::Success
    }
}
