struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec4<f32>,
    @location(3) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) tex_coords: vec2<f32>,
    @location(4) norm_mat_0: vec4<f32>,
    @location(5) norm_mat_1: vec4<f32>,
    @location(6) norm_mat_2: vec4<f32>,
    @location(7) norm_mat_3: vec4<f32>,
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
@group(1) @binding(0)
var<uniform> p_light: PointLight;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(2)
var t_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;

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
    out.tex_coords = in.tex_coords;
    out.norm_mat_0 = data.normal_mat_0;
    out.norm_mat_1 = data.normal_mat_1;
    out.norm_mat_2 = data.normal_mat_2;
    out.norm_mat_3 = data.normal_mat_3;
    let u = in.position.x;
    let v = in.position.y;
    let new_pos = vec4<f32>(plane_func(u, v), 1.0);
    let neigh1 = plane_func(u + 0.001, v);
    let neigh2 = plane_func(u, v + 0.001);
    let tangent = normalize(neigh1 - new_pos.xyz);
    let bitangent = normalize(neigh2 - new_pos.xyz);
    let norm = normalize(cross(tangent, bitangent));

    let world_position = model * new_pos;
    out.world_position = world_position.xyz;

    out.frag_position = camera.projection * camera.view * model * new_pos;
    // out.world_normal = (plane_entity_data.normal_mat * in.normal).xyz;
    out.world_normal = (norm_mat * vec4<f32>(norm, 1.0)).xyz;
    return out;
}


fn plane_func(u: f32, v: f32) -> vec3<f32> {
    let z = sin(u * 10.0 * 3.1415) * 0.01 + sin(v * 5.0 * 3.1415) * 0.05;
    return vec3<f32>(u, v, z);
}


// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let obj_norm: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);
    let norm_mat = mat4x4<f32>(
        in.norm_mat_0,
        in.norm_mat_1,
        in.norm_mat_2,
        in.norm_mat_3,
    );
    var norm = normalize((norm_mat * obj_norm * 2.0 - 1.0).xyz);


    let ambient_strength = 0.1;
    let ambient_col = p_light.diffuse_color.rgb * ambient_strength;

    let light_vec = (p_light.position).xyz - in.world_position;
    let light_dir = normalize(light_vec.xyz);
    let view_dir = normalize(camera.position.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, norm.xyz);

    let dist = length(light_vec.xyz);
    let atten = 1.0 / (max(0.01, dist) * max(0.01, dist));
    let diffuse_strength = max(dot(norm.xyz, light_dir), 0.0) * atten;
    let diff_color = p_light.diffuse_color.rgb * diffuse_strength;

    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32) * 0.5;
    let specular_color = p_light.diffuse_color.rgb * specular_strength;

    let result = vec4<f32>((ambient_col + diff_color + specular_color) * col.rgb, 1.0);

    return result;
}

