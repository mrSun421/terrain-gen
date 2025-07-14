use bytemuck::{Pod, Zeroable};
use cgmath::*;
pub struct PointLight {
    pub position: Vector4<f32>,
    ambient_color: Vector3<f32>,
}

impl PointLight {
    pub fn new(position: Vector4<f32>, ambient_color: Vector3<f32>) -> Self {
        Self {
            position,
            ambient_color,
        }
    }

    pub fn to_uniform_data(&self) -> PointLightUniformData {
        let pos = self.position.into();
        let diffuse_color = Point3::from_vec(self.ambient_color).to_homogeneous().into();
        PointLightUniformData { pos, diffuse_color }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PointLightUniformData {
    pos: [f32; 4],
    diffuse_color: [f32; 4],
}
