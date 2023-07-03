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

    @location(0) uv: vec2<f32>,
    @location(1) centroid_world_position: vec4<f32>,
    @location(2) centroid_clip_position: vec4<f32>,
};


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
        }
    }

    var vert0_world = mesh_position_local_to_world(mesh.model, vec4(vert0.position, 1.0)) / 2.0;
    var vert1_world = mesh_position_local_to_world(mesh.model, vec4(vert1.position, 1.0)) / 2.0;
    var vert2_world = mesh_position_local_to_world(mesh.model, vec4(vert2.position, 1.0)) / 2.0;

    out.centroid_world_position = (vert0_world + vert1_world + vert2_world) / 3.0;
    out.centroid_clip_position = mesh_position_world_to_clip(out.centroid_world_position);
    out.uv = current_vert.uv;

    return out;
}


struct InterpolatedFragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

    @location(0) uv: vec2<f32>,
    @location(1) centroid_world_position: vec4<f32>,
    @location(2) centroid_clip_position: vec4<f32>,
};

@fragment
fn fragment(in: InterpolatedFragmentInput) -> @location(0) vec4<f32> {
    var output_color: vec4<f32>;

    var sphere_center = in.centroid_clip_position;

    var viewport_uv = coords_to_viewport_uv(in.frag_coord.xy, view.viewport);
    viewport_uv *= 2.0;
    viewport_uv -= 1.0;
    // adjust for aspect ratio of the viewport, I guess? this seems to look more correct
    viewport_uv.x /= view.viewport.w / view.viewport.z;

    // orthographic projection. perspective would project from a single point as origin
    var ray_origin = vec3(viewport_uv, 1.0);
    var ray_direction = vec3(viewport_uv, 0.0) - ray_origin.xyz;

    // distance from the sphere center to the ray
    var dist = length(cross(sphere_center.xyz - ray_origin.xyz, ray_direction.xyz)) / length(ray_direction.xyz);

    // TODO: need to z-order the spheres somehow, maybe with a depth prepass or something?

    // TODO PBR rendering. oof it's probably gonna be expensive
    output_color = textureSample(emissive_texture, emissive_sampler, in.uv);

    if dist > bubble_radius {
        discard;
    }

    return output_color;
}
