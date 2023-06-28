// mostly a copy of `bevy_pbr::mesh`

#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

@group(1) @binding(100)
var<uniform> noise_magnitude: f32;

@group(1) @binding(101)
var<uniform> noise_scale: f32;

// @group(1) @binding(102)
// var<uniform> time: f32;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};


// https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39
fn mod289(x: vec4<f32>) -> vec4<f32> { return x - floor(x * (1. / 289.)) * 289.; }
fn perm4(x: vec4<f32>) -> vec4<f32> { return mod289(((x * 34.) + 1.) * x); }

// TODO: double check what the output range of this is... Seems subtracting one
// might tend towards small
fn noise3(p: vec3<f32>) -> f32 {
  let a = floor(p);
  var d: vec3<f32> = p - a;
  d = d * d * (3. - 2. * d);

  let b = a.xxyy + vec4<f32>(0., 1., 0., 1.);
  let k1 = perm4(b.xyxy);
  let k2 = perm4(k1.xyxy + b.zzww);

  let c = k2 + a.zzzz;
  let k3 = perm4(c);
  let k4 = perm4(c + 1.);

  let o1 = fract(k3 * (1. / 41.));
  let o2 = fract(k4 * (1. / 41.));

  let o3 = o2 * d.z + o1 * (1. - d.z);
  let o4 = o3.yw * d.x + o3.xz * (1. - d.x);

  return o4.y * d.y + o4.x * (1. - d.y);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var world_normal = mesh_normal_local_to_world(vertex.normal);

    var world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));

    var offset = noise_magnitude * (
        // TODO: noise4 with globals.time might look nice here
        noise3(vertex.position.xyz * noise_scale) - 1.0
    );
    // TODO: random direction instead of normal? It actually looks decent like this already!
    world_position += vec4(offset * world_normal, 0.0);

    out.world_position = world_position;
    out.world_normal = world_normal;
    out.clip_position = mesh_position_world_to_clip(world_position);

    out.uv = vertex.uv;

    return out;
}
