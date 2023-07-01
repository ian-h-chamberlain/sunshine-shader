//! Sketch of what a custom pipeline might look like, to plumb the mesh vertex buffer
//! into the material shader as a buffer uniform.

use bevy::ecs::system::{lifetimeless::*, SystemParamItem};
use bevy::pbr::{MeshUniform, RenderMaterials, SetMeshBindGroup, SetMeshViewBindGroup};
use bevy::prelude::*;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_asset::*;
use bevy::render::render_phase::*;
use bevy::render::render_resource::{
    AsBindGroup, BindGroupDescriptor, BindGroupEntry, OwnedBindingResource,
};
use bevy::render::renderer::RenderDevice;

use super::BubblesMaterial;

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    // SetMaterialBindGroup<BubblesMaterial, 1>, // skipped because we set the bind group in `Draw`
    SetMeshBindGroup<2>,
    Draw,
);

pub struct Draw;

/// Rename -> Bubbles and use for extending StandardMaterial
#[derive(AsBindGroup)]
struct CoolMaterial {
    #[uniform(100)]
    pub bubble_radius: f32,

    #[storage(101, read_only)]
    values: Vec<Vec3>,
}

impl<P: PhaseItem> RenderCommand<P> for Draw {
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderAssets<Mesh>>,
        SRes<RenderMaterials<BubblesMaterial>>,
        SRes<Assets<BubblesMaterial>>,
    );

    type ViewWorldQuery = ();

    type ItemWorldQuery = (
        Read<Handle<Mesh>>,
        Read<Handle<BubblesMaterial>>,
        Read<DynamicUniformIndex<MeshUniform>>,
    );

    fn render<'w>(
        item: &P,
        view: (),
        (mesh_handle, material_handle, mesh_uniform): (
            &'w Handle<Mesh>,
            &'w Handle<BubblesMaterial>,
            &'w DynamicUniformIndex<MeshUniform>,
        ),
        (render_device, meshes, prepared_materials, materials): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (Some(prepared_material), Some(material), Some(mesh)) = (
            prepared_materials.get(material_handle),
            materials.get(material_handle),
            meshes.get(mesh_handle),
        ) else { return RenderCommandResult::Failure };

        let mut entries = Vec::new();

        for (binding, resource) in &prepared_material.bindings {
            let resource = match resource {
                // we know the "owned" buffer resources are bogus, so
                OwnedBindingResource::Buffer(buf) => {
                    OwnedBindingResource::Buffer(mesh.vertex_buffer)
                }
                OwnedBindingResource::TextureView(view) => {
                    OwnedBindingResource::TextureView(view.clone())
                }
                OwnedBindingResource::Sampler(sampler) => {
                    OwnedBindingResource::Sampler(sampler.clone())
                }
            };

            entries.push(BindGroupEntry {
                binding: *binding,
                resource: resource.get_binding(),
            });
        }

        // This stuff maybe could be in a prepare/extract stage, this might be kinda hot
        // to be doing stuff like this tbh
        let layout = CoolMaterial::bind_group_layout(&render_device);

        let material_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None, // TODO
            layout: &layout,
            entries: &entries,
        });

        pass.set_bind_group(1, &material_bind_group, &[mesh_uniform.index()]);

        // TODO:
        // pass.draw(vertices, instances);

        RenderCommandResult::Success
    }
}
