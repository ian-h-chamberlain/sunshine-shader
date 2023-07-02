#import bevy_pbr::pbr_fragment

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@group(1) @binding(100)
var<uniform> bubble_radius: f32;

@group(1) @binding(101)
var<storage> vertex_buffer: array<Vertex>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) @interpolate(flat) uv: vec2<f32>,

    @location(3) @interpolate(flat) vert0: vec4<f32>,
    @location(4) @interpolate(flat) vert1: vec4<f32>,
    @location(5) @interpolate(flat) vert2: vec4<f32>,

    @location(6) @interpolate(flat) vert_idx: u32,
};


// Simple vertex shader (basically same as `bevy_pbr::mesh.wgsl`) but with an
// extra interpolated output for face position
@vertex
fn vertex(
    @builtin(instance_index) instance_index: u32,
    @location(0) quad_vert_position: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    var current_vert: Vertex;
    var vert0: Vertex;
    var vert1: Vertex;
    var vert2: Vertex;

    // the quad verts are already positioned in clip space. EZPZ
    out.clip_position = vec4(quad_vert_position, 1.0);

    out.vert_idx = instance_index;
    switch instance_index % 3u {
        case 0u: {
            vert0 = vertex_buffer[instance_index];
            vert1 = vertex_buffer[instance_index + 1u];
            vert2 = vertex_buffer[instance_index + 2u];

            current_vert = vert0;
        }
        case 1u: {
            vert0 = vertex_buffer[instance_index - 1u];
            vert1 = vertex_buffer[instance_index];
            vert2 = vertex_buffer[instance_index + 1u];

            current_vert = vert1;
        }
        case 2u: {
            vert0 = vertex_buffer[instance_index - 2u];
            vert1 = vertex_buffer[instance_index - 1u];
            vert2 = vertex_buffer[instance_index];

            current_vert = vert2;
        }
        default: {
            // definitely impossible, right??
            vert0.position = vec3(1.0, 0.0, 0.0);
            vert1.position = vec3(0.0, 1.0, 0.0);
            vert2.position = vec3(0.0, 0.0, 1.0);
        }
    }

    out.world_position = mesh_position_local_to_world(mesh.model, vec4(current_vert.position, 1.0));

    out.vert0 = vec4(vert0.position, 1.0);
    out.vert1 = vec4(vert1.position, 1.0);
    out.vert2 = vec4(vert2.position, 1.0);

    // out.vert0 = mesh_position_local_to_world(mesh.model, vec4(vert0.position, 1.0));
    // out.vert1 = mesh_position_local_to_world(mesh.model, vec4(vert1.position, 1.0));
    // out.vert2 = mesh_position_local_to_world(mesh.model, vec4(vert2.position, 1.0));

    // out.uv = vertex.uv;

    return out;
}


struct InterpolatedFragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) @interpolate(flat) uv: vec2<f32>,

    @location(3) @interpolate(flat) vert0: vec4<f32>,
    @location(4) @interpolate(flat) vert1: vec4<f32>,
    @location(5) @interpolate(flat) vert2: vec4<f32>,

    @location(6) @interpolate(flat) vert_idx: u32,
};

@fragment
fn fragment(in: InterpolatedFragmentInput) -> @location(0) vec4<f32> {
    var pbr_in: FragmentInput;

    var output_color: vec4<f32>;

    // this code attempts to SDF a sphere, but it won't work by using
    // centroid as the center of the sphere, since that varies per
    // primitive (or per fragment without interpolate(flat),
    // and really we need to do this once per vertex. Maybe we can do some kinda
    // instancing hack where we re-feed the VBO as instance data, to run this
    // fragment shader once per vertex?
    //
    // It will probably get expensive fast tho lol. Might need a prepass shader
    // that does some stuff up front.
    //
    // Ooh also compute shaders are a thing I forgot about. Maybe worth looking into

    var centroid = (in.vert0 + in.vert1 + in.vert1) / 3.0;

    if 1.0 > 0.0 {
        return vec4(
            in.vert0,
        );
    }

    // var centroid_world_pos = mesh_position_local_to_world(mesh.model, vec4(centroid, 1.0));

    // if length(centroid) > 0.0 {
    //     return vec4(1.0, 0.0, 1.0 * distance(centroid_world_pos, in.world_position), 1.0);
    // }

    var viewport_uv = coords_to_viewport_uv(in.frag_coord.xy, view.viewport).xy;
    viewport_uv *= 2.0;
    viewport_uv -= 1.0;

    var ray_direction = normalize(vec3(viewport_uv, 1.0));

    var distance: f32;

    var ray_start = vec3(0.0);
    var radius_sq = bubble_radius * bubble_radius;
    var dt = dot(ray_direction, centroid.xyz - ray_start);
    if dt < 0.0 {
        return vec4(0.0);
    }

    var tmp = ray_start - centroid.xyz;
    tmp.x = dot(tmp, tmp);
    tmp.x = tmp.x - dt * dt;
    if tmp.x >= bubble_radius {
        return vec4(0.0);
    }

    dt = dt - sqrt(bubble_radius - tmp.x);
    var point = ray_start + ray_direction * dt;
    var normal = normalize(point - centroid.xyz);

    if dt > 0.0 {
        // TODO PBR rendering of the sphere
        output_color = vec4(1.0, 0.0, 1.0, 1.0);
    }

    // pbr_in.is_front = in.is_front;
    // pbr_in.frag_coord = in.frag_coord;
    // pbr_centroid = centroid;
    // pbr_in.uv = in.uv;

    // // TODO: adjust normal
    // pbr_in.world_normal = in.world_normal;

    // var output_color: vec4<f32>;


    // // not really sure why we have to do this, but convert from (-1,-1)↗(1,1) to (0,0)↘(1,1)
    // // and we also have to manually apply the perspective divisor (w).
    // var interp_viewport_uv = vec2(0.5, -0.5) * (in.triangle_centroid.xy / in.triangle_centroid.w) + 0.5;

    // var dist = // 1.0;
    // distance(viewport_uv, interp_viewport_uv);

    // if dist < bubble_radius {
    //     // call to the standard pbr fragment shader
    //     output_color = pbr_fragment(pbr_in);
    //     // output_color = vec4(
    //     //     interp_viewport_uv.x,
    //     //     0.0,
    //     //     interp_viewport_uv.y,
    //     //     1.0,
    //     // );
    // } else {
    //     // output_color = vec4(
    //     //     in.frag_coord.x / in.frag_coord.w,
    //     //     0.0,
    //     //     in.frag_coord.y / in.frag_coord.w,
    //     //     1.0,
    //     // );
    //     discard;
    // }

    return output_color;
}
