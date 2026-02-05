// Animated starfield background shader
// Uses Bevy's Globals for time, and a custom uniform for resolution.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BackgroundMaterial {
    time: f32,
    resolution: vec2<f32>,
    _padding: f32,
}

@group(2) @binding(0) var<uniform> material: BackgroundMaterial;

// Hash function for pseudo-random star placement
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

// Smooth noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = material.time;

    // Background gradient: dark navy at top, dark purple at bottom
    let bg_top = vec3<f32>(0.02, 0.02, 0.08);
    let bg_bottom = vec3<f32>(0.06, 0.02, 0.08);
    var color = mix(bg_top, bg_bottom, uv.y);

    // Subtle nebula wisps using layered noise
    let nebula_uv = uv * 3.0 + vec2<f32>(time * 0.02, time * 0.01);
    let n = noise(nebula_uv) * 0.5 + noise(nebula_uv * 2.0) * 0.25;
    let nebula_color = vec3<f32>(0.1, 0.05, 0.15) * n * 0.4;
    color += nebula_color;

    // Stars: grid-based placement with hash for position and brightness
    let star_grid = uv * 40.0;
    let star_cell = floor(star_grid);
    let star_fract = fract(star_grid);

    let star_pos = vec2<f32>(hash(star_cell), hash(star_cell + vec2<f32>(17.0, 31.0)));
    let star_dist = length(star_fract - star_pos);

    let star_brightness = hash(star_cell + vec2<f32>(59.0, 83.0));
    let twinkle = sin(time * (1.0 + star_brightness * 3.0) + star_brightness * 6.28) * 0.3 + 0.7;
    let star_size = 0.03 + star_brightness * 0.02;

    if star_dist < star_size && star_brightness > 0.6 {
        let star_intensity = (1.0 - star_dist / star_size) * twinkle;
        let star_color = mix(
            vec3<f32>(0.8, 0.85, 1.0),
            vec3<f32>(1.0, 0.9, 0.7),
            star_brightness
        );
        color += star_color * star_intensity * 0.8;
    }

    return vec4<f32>(color, 1.0);
}
