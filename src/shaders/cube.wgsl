struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec4<f32>,
    @location(3) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};

struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct PointLight {
    position: vec4<f32>,
    diffuse_color: vec4<f32>,
};

struct EntityData {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) normal_mat_0: vec4<f32>,
    @location(9) normal_mat_1: vec4<f32>,
    @location(10) normal_mat_2: vec4<f32>,
    @location(11) normal_mat_3: vec4<f32>,
};


@vertex
fn vs_main(
    in: VertexInput,
    data: EntityData,
) -> VertexOutput {
    let model = mat4x4<f32>(
        data.model_0,
        data.model_1,
        data.model_2,
        data.model_3,
    );
    let norm_mat = mat4x4<f32>(
        data.normal_mat_0,
        data.normal_mat_1,
        data.normal_mat_2,
        data.normal_mat_3,
    );

    var out: VertexOutput;
    out.color = in.color;

    out.position = camera.projection * camera.view * model * in.position;
    out.normal = (norm_mat * in.normal).xyz;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let norm: vec3<f32> = normalize(in.normal.xyz);

    var result: vec4<f32> = in.color;

    return result;
}


