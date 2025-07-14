use bytemuck::{Pod, Zeroable};
use cgmath::*;
use wgpu::{util::DeviceExt, vertex_attr_array};

use crate::vertex::{Mesh, MeshData};

pub struct EntityWrapper {
    pub entity: Entity,
    pub mesh_data: MeshData,
}

impl EntityWrapper {
    pub fn new(entity: Entity, device: &wgpu::Device) -> Self {
        let mesh_data = entity.mesh.to_mesh_data(device);
        Self { entity, mesh_data }
    }

    pub fn update_entity_position(&mut self, pos: cgmath::Vector3<f32>) {
        self.entity.position = pos;
    }

    pub fn to_entity_data(&self) -> EntityData {
        self.entity.to_entity_data()
    }
}
pub struct Entity {
    mesh: Mesh,
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    scale: cgmath::Vector3<f32>,
}
impl Entity {
    pub fn new(
        mesh: Mesh,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        scale: cgmath::Vector3<f32>,
    ) -> Self {
        Self {
            mesh,
            position,
            rotation,
            scale,
        }
    }

    fn get_model_matrix(&self) -> cgmath::Matrix4<f32> {
        let mut model = cgmath::Matrix4::identity();
        model = model * cgmath::Matrix4::from_translation(self.position);
        model = model
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        model = model * cgmath::Matrix4::from(self.rotation);

        model
    }
    pub fn get_normal_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from(self.rotation)
    }

    pub fn to_entity_data(&self) -> EntityData {
        EntityData {
            mx_model: self.get_model_matrix().into(),
            mx_normal: self.get_normal_matrix().into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct EntityData {
    mx_model: [[f32; 4]; 4],
    mx_normal: [[f32; 4]; 4],
}

impl EntityData {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<EntityData>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &vertex_attr_array![
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4,
            9 => Float32x4,
            10 => Float32x4,
            11 => Float32x4,
        ],
    };
}
