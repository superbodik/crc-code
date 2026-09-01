struct Globals {
    screen: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct Instance {
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) border_color: vec4<f32>,
    @location(3) radius_border: vec2<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) radius_border: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32, instance: Instance) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let corner = corners[index];
    let size = instance.rect.zw;
    let pixel = instance.rect.xy + corner * size;

    var out: VertexOut;
    out.position = vec4<f32>(
        pixel.x / globals.screen.x * 2.0 - 1.0,
        1.0 - pixel.y / globals.screen.y * 2.0,
        0.0,
        1.0,
    );
    out.local = corner * size;
    out.size = size;
    out.color = instance.color;
    out.border_color = instance.border_color;
    out.radius_border = instance.radius_border;
    return out;
}

fn rounded_box(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(point) - half_size + vec2<f32>(radius, radius);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let half_size = in.size * 0.5;
    let point = in.local - half_size;

    let radius = min(in.radius_border.x, min(half_size.x, half_size.y));
    let border = in.radius_border.y;

    let distance = rounded_box(point, half_size, radius);
    let coverage = 1.0 - smoothstep(-0.5, 0.5, distance);
    if coverage <= 0.0 {
        discard;
    }

    var color = in.color;
    if border > 0.0 {
        let inside = 1.0 - smoothstep(-0.5, 0.5, distance + border);
        color = mix(in.border_color, in.color, inside);
    }

    return vec4<f32>(color.rgb, color.a * coverage);
}
