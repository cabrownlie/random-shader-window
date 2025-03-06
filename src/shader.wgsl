// A shader for messing around with signed distance fields and ray marching.

// This is used to pass the mouse position to the shader
@group(0) @binding(0) var<uniform> point : vec2<f32>;

struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

struct FragmentIn {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2f,
};

struct Ray {
    origin: vec2<f32>,
    direction: vec2<f32>,
};

struct HitResult {
    distance: f32,
    position: vec2<f32>,
}

// The vertex shader just creates a quad that covers the whole screen since we will be doing sdf ray marching stuff.
// The fragment shader will do the heavy lifting
@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2(1.0, 1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(-1.0, 1.0),
    );

    var uv = array<vec2<f32>, 6>(
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
    );

    var output: VertexOutput;
    output.pos = vec4(pos[index], 0.0, 1.0);
    output.uv = uv[index];
    return output;
}

@fragment
fn fragment(in: FragmentIn) -> @location(0) vec4<f32> {
    var colour = vec3(in.uv, 0.0); // Default background colour
    var base_colour = vec3(1.0, 0.0, 1.0); // Default colour objects colour
    var shadow_colour = vec3(0.0, 0.0, 0.0); // Colour for occluded areas

    let ray = Ray(point, normalize(in.pos.xy - point));
    let pixel_distance = distance(in.pos.xy, point);
    let max_distance_to_fade = 100.0; // The maximum distance from the surface where the fade effect should complete

    // If the current pixel is inside the scene objects, set the colour to the base colour
    if scene(in.pos.xy) < 0.0 {
        colour = base_colour;
    }

    // Perform ray marching to find intersection with the scene
    var hit_info = ray_march(ray.origin, ray.direction, pixel_distance);
    if hit_info.distance < pixel_distance {
        colour = mix(colour, shadow_colour, 0.8); // Mix the base colour with the shadow colour
    } else {
        // If we didn't hit an object, fade out the colour based on the distance to the scene
        let distance = 1.0 - (scene(point) / hit_info.distance);
        colour += mix(colour, shadow_colour, distance);
    }

    // Draw a circle at the 'point' location
    if circle(in.pos.xy, point, 5.) < 0.0 {
        colour = vec3(1.0, 1.0, 0.0); // Yellow color for the point circle
    }

    return vec4(colour, 1.0);
}

fn ray_march(origin: vec2<f32>, direction: vec2<f32>, max_distance: f32) -> HitResult {
    // The threshold value is the minimum distance we want to be from the surface
    let threshold = 0.01;

    // Ray march up to the max_distance
    var step = 0.0;
    while step < max_distance {
        let dist = scene(origin + direction * step);
        if dist < threshold {
            // We hit an object before reaching the max_distance, so it's occluded
            return HitResult(dist, origin + direction * step);
        }
        // Move the step ahead by the returned scene distance
        step += dist;
    }

    return HitResult(max_distance, origin);
}

// The scene function returns the distance to the closest object in the scene
// TODO: Grab object data from the buffer instead of hardcoding in the shader.
fn scene(pos: vec2<f32>) -> f32 {
    let c1 = circle(pos, vec2(200., 200.), 50.);
    let c2 = circle(pos, vec2(400., 300.), 30.);
    let b1 = box(pos, vec2(600., 300.), vec2(50., 50.));
    return min(min(c1, c2), b1);
}


fn circle(pos: vec2<f32>, centre: vec2<f32>, radius: f32) -> f32 {
    return distance(pos, centre) - radius;
}

fn box(pos: vec2<f32>, centre: vec2<f32>, size: vec2<f32>) -> f32 {
    let dist = abs(pos - centre) - size / 2.;
    return length(max(dist, vec2(0., 0.))) + min(max(dist.x, dist.y), 0.0);
}

fn calculate_normal(pos: vec2<f32>) -> vec2<f32> {
    let epsilon = vec2<f32>(0.001, 0.0);
    let normal = normalize(vec2<f32>(
        scene(pos + epsilon.xy) - scene(pos - epsilon.xy),
        scene(pos + epsilon.yy) - scene(pos - epsilon.yy)
    ));

    return normal;
}
