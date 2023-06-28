use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::reflect::TypeUuid;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::{log, prelude::*};

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "eee72aef-5111-4307-a571-191b80a73dbe"]
pub struct NoisyVertMaterial {
    /// How far (at most) offset vertices should be (in ??? units)
    #[uniform(100)]
    pub noise_magnitude: f32,

    /// The scale of the noize
    #[uniform(101)]
    pub noise_scale: f32,
}

impl NoisyVertMaterial {
    pub fn new(noise_magnitude: f32, noise_scale: f32) -> Self {
        Self {
            noise_magnitude,
            noise_scale,
        }
    }
}

impl Material for NoisyVertMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/noisy_vert.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(label) = &mut descriptor.label {
            *label = format!("noisy_{label}").into();
        }

        log::debug!("vert buffers: {:#?}", descriptor.vertex.buffers);

        Ok(())
    }
}
