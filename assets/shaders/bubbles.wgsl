#import bevy_pbr::pbr_fragment

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions


@group(1) @binding(100)
var<uniform> bubble_radius: f32;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(10) triangle_centroid: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) @interpolate(flat) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) @interpolate(flat) uv: vec2<f32>,
    @location(3) triangle_centroid: vec4<f32>,
};


// Simple vertex shader (basically same as `bevy_pbr::mesh.wgsl`) but with an
// extra interpolated output for face position
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.clip_position = mesh_position_world_to_clip(out.world_position);

    out.uv = vertex.uv;

    // TODO: maybe some kind of scaling based on how big the triangle is...
    var world_centroid = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.triangle_centroid, 1.0));
    out.triangle_centroid = mesh_position_world_to_clip(world_centroid);

    return out;
}


struct InterpolatedFragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,

    @location(0) @interpolate(flat) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) @interpolate(flat) uv: vec2<f32>,
    @location(3) triangle_centroid: vec4<f32>,
};

@fragment
fn fragment(in: InterpolatedFragmentInput) -> @location(0) vec4<f32> {
    var pbr_in: FragmentInput;

    var output_color: vec4<f32>;

    // this code attempts to SDF a sphere, but it won't work by using
    // in.world_position as the center of the sphere, since that varies per
    // primitive (or per fragment without interpolate(flat),
    // and really we need to do this once per vertex. Maybe we can do some kinda
    // instancing hack where we re-feed the VBO as instance data, to run this
    // fragment shader once per vertex?
    //
    // It will probably get expensive fast tho lol. Might need a prepass shader
    // that does some stuff up front.
    //
    // Ooh also compute shaders are a thing I forgot about. Maybe worth looking into


    var viewport_uv = coords_to_viewport_uv(in.frag_coord.xy, view.viewport).xy;
    viewport_uv *= 2.0;
    viewport_uv -= 1.0;

    var ray_direction = normalize(vec3(viewport_uv, 1.0));

    var distance: f32;

    var ray_start = vec3(0.0);
	var radius_sq = bubble_radius * bubble_radius;
	var dt = dot(ray_direction, in.world_position.xyz - ray_start);
	if (dt < 0.0) {
		return vec4(0.0);
	}

	var tmp = ray_start - in.world_position.xyz;
	tmp.x = dot(tmp, tmp);
	tmp.x = tmp.x - dt*dt;
	if (tmp.x >= bubble_radius) {
		return vec4(0.0);
	}

	dt = dt - sqrt(bubble_radius - tmp.x);
	var point = ray_start + ray_direction * dt;
	var normal = normalize(point - in.world_position.xyz);

    if dt > 0.0 {
        // TODO PBR rendering of the sphere
        output_color = vec4(1.0, 0.0, 1.0, 1.0);
    }

    // pbr_in.is_front = in.is_front;
    // pbr_in.frag_coord = in.frag_coord;
    // pbr_in.world_position = in.world_position;
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
