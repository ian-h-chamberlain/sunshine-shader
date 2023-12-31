use bevy::log;
use bevy::pbr::{ExtendedMaterial, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

pub type NoisyVertsMaterial = ExtendedMaterial<NoisyVerts>;

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "eee72aef-5111-4307-a571-191b80a73dbe"]
pub struct NoisyVerts {
    /// How far (at most) offset vertices should be (in ??? units)
    #[uniform(100)]
    pub noise_magnitude: f32,

    /// The scale of the noize
    #[uniform(101)]
    pub noise_scale: f32,

    /// The speed at which the shader should animate
    #[uniform(102)]
    pub time_scale: f32,
}

impl Default for NoisyVerts {
    fn default() -> Self {
        Self {
            noise_magnitude: 1.0,
            noise_scale: 1.0,
            time_scale: 1.0,
        }
    }
}

impl Material for NoisyVerts {
    fn vertex_shader() -> ShaderRef {
        "shaders/noisy_verts.wgsl".into()
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
