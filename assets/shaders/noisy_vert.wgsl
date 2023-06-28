// mostly a copy of `bevy_pbr::mesh`

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

@group(1) @binding(0)
var<uniform> noise_level: f32;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
    // NOTE: this has to be done _after_ the world_position is calculated!!! duh!!!
    out.clip_position = mesh_position_world_to_clip(out.world_position);

    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.uv = vertex.uv;

    return out;
}
