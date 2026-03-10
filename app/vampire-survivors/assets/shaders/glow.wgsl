// Radial glow ring shader for treasure chests and other collectibles.
//
// The mesh is a quad; UV (0,0)–(1,1) maps the full quad.
// The glow is a ring between [inner_radius, outer_radius] in UV-distance space,
// brightest at the inner edge and fading to zero at the outer edge.
// A gentle brightness pulse is driven by globals.time (GPU-driven, no CPU work).

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct GlowParams {
    glow_color:   vec4<f32>,
    inner_radius: f32,
    outer_radius: f32,
    _pad:         vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: GlowParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let dist   = distance(in.uv, center);

    // Gentle brightness pulse (0.70 – 1.00) at ~3 rad/s.
    let pulse = 0.85 + 0.15 * sin(globals.time * 3.0);

    // Radial falloff:
    //   dist < inner_radius  → 0 (transparent — the chest body occupies this area)
    //   inner_radius .. outer_radius → 1 → 0 (bright ring fading outward)
    //   dist > outer_radius  → 0 (transparent outside the mesh)
    let ring = smoothstep(material.outer_radius, material.inner_radius, dist)
             * step(material.inner_radius, dist);

    let alpha = ring * pulse * material.glow_color.a;
    return vec4<f32>(material.glow_color.rgb, alpha);
}
