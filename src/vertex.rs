use bytemuck::{Pod, Zeroable};
use cgmath::*;
use wgpu::{util::DeviceExt, vertex_attr_array};

use crate::texture;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    pub normal: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x2,
        ],
    };
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn to_mesh_data(&self, device: &wgpu::Device) -> MeshData {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let index_len = self.indices.len() as u32;

        MeshData {
            vertex_buf,
            index_buf,
            index_len,
        }
    }

    pub fn generate_plane(&mut self, resolution: u32) {
        let resolution_recip = 1.0 / resolution as f32;
        let vertices: Vec<cgmath::Point3<f32>> = (0..(resolution + 1))
            .flat_map(|x| {
                (0..(resolution + 1)).map(move |y| {
                    cgmath::point3(
                        x as f32 * resolution_recip,
                        y as f32 * resolution_recip,
                        0.0,
                    )
                })
            })
            .collect();
        let indices: Vec<u32> = (0..resolution)
            .flat_map(|x| {
                (0..resolution).flat_map(move |y| {
                    let idx_0 = y * (resolution + 1) + x;
                    let idx_1 = idx_0 + 1;
                    let idx_2 = (y + 1) * (resolution + 1) + x;
                    let idx_3 = idx_2 + 1;
                    vec![idx_0, idx_2, idx_1, idx_2, idx_3, idx_1]
                })
            })
            .collect();
        let vertices_raw: Vec<Vertex> = vertices
            .into_iter()
            .map(|v| Vertex {
                pos: v.to_homogeneous().into(),
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                tex_coords: v.xy().into(),
            })
            .collect();
        self.vertices.extend(vertices_raw.iter());
        self.indices.extend(indices);
    }

    pub fn generate_cube(&mut self) {
        let vertex_data = [
            Vertex {
                pos: [-1.0, -1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                pos: [1.0, -1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                pos: [-1.0, 1.0, -1.0, 1.0],
                normal: [0.0, 0.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, -1.0, 1.0],
                normal: [0.0, 0.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                pos: [1.0, -1.0, -1.0, 1.0],
                normal: [0.0, 0.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                pos: [-1.0, -1.0, -1.0, 1.0],
                normal: [0.0, 0.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, -1.0, 1.0],
                normal: [1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, -1.0, 1.0],
                normal: [1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0, 1.0, 1.0],
                normal: [1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, 1.0, 1.0],
                normal: [1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                pos: [-1.0, -1.0, 1.0, 1.0],
                normal: [-1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 1.0, 1.0],
                normal: [-1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, -1.0, 1.0],
                normal: [-1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                pos: [-1.0, -1.0, -1.0, 1.0],
                normal: [-1.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0, -1.0, 1.0],
                normal: [0.0, 1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, -1.0, 1.0],
                normal: [0.0, 1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, 1.0, 1.0],
                normal: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, -1.0, 1.0, 1.0],
                normal: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                pos: [-1.0, -1.0, -1.0, 1.0],
                normal: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0, -1.0, 1.0],
                normal: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
        ];

        let index_data: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];
        self.vertices.extend(vertex_data);
        self.indices.extend(index_data);
    }
}

pub struct MeshData {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_len: u32,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
        });
        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}
