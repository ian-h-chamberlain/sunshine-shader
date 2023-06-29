use bevy::pbr::{ExtendedMaterial, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

pub type BubblesMaterial = ExtendedMaterial<Bubbles>;

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "68c25f8b-b16a-4630-aa6c-e0399e71fbd6"]
pub struct Bubbles {
    /// How big the bubbles should be
    #[uniform(100)]
    pub bubble_radius: f32,
    // TODO: probably some options about thresholds / quantization / idk
}

impl Default for Bubbles {
    fn default() -> Self {
        Self { bubble_radius: 1.0 }
    }
}

impl Material for Bubbles {
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

        Ok(())
    }
}
