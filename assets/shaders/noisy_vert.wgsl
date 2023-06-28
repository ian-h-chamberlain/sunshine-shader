// mostly a copy of `bevy_pbr::mesh`

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

#import "shaders/perlin.wgsl"

@group(1) @binding(100)
var<uniform> noise_magnitude: f32;

@group(1) @binding(101)
var<uniform> noise_scale: f32;


@group(1) @binding(102)
var<uniform> time_scale: f32;

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

    var world_normal = mesh_normal_local_to_world(vertex.normal);

    var world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));

    var noise_sample = vec4(noise_scale * vertex.position.xyz, globals.time * time_scale);
    // TODO: noise offset, or abs(snoise(...)) ? It might be nice to avoid pushing verts
    // in away since this sometimes causes a weird overlap effect that doesn't look super pretty

    var offset = noise_magnitude * snoise(noise_sample);
    // TODO: random direction instead of normal? It actually looks decent like this already!
    world_position += vec4(offset * world_normal, 0.0);

    out.world_position = world_position;
    out.world_normal = world_normal;
    out.clip_position = mesh_position_world_to_clip(world_position);

    out.uv = vertex.uv;

    return out;
}
