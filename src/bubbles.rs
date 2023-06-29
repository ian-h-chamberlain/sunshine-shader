// TODO: investigate instancing as an option - since we already have all the
// vertex data it might be feasible to to do without any extra buffers...
// <https://bevyengine.org/examples/shader/shader-instancing/>

use bevy::log;
use bevy::pbr::{ExtendedMaterial, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayout};
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, VertexFormat,
};

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

pub const ATTRIBUTE_TRIANGLE_CENTROID: MeshVertexAttribute =
    MeshVertexAttribute::new("TriangleCentroid", 1102843625, VertexFormat::Float32x3);

impl Material for Bubbles {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

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

        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TRIANGLE_CENTROID.at_shader_location(10),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}
