#import bevy_pbr::pbr_fragment

@group(1) @binding(100)
var<uniform> bubble_radius: f32;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // TODO actually implement some shit here

    // call to the standard pbr fragment shader
    var output_color = pbr_fragment(in);

    return output_color;
}
