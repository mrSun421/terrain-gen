use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::Duration,
};

use bytemuck::{Pod, Zeroable};
use cgmath::{num_traits::clamp, *};
use winit::{event::ElementState, keyboard::KeyCode};
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[derive(Debug)]
pub struct Camera {
    position: Vector3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
    world_up: Vector3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    fovy: Deg<f32>,
}

impl Camera {
    fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(Point3::from_vec(self.position), self.front, self.up)
    }

    fn get_projection_matrix(&self, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, aspect, znear, zfar)
    }
}
impl Default for Camera {
    fn default() -> Self {
        let position = Vector3::zero();
        let front = -Vector3::unit_z();
        let up = Vector3::unit_y();
        let world_up = up;
        let yaw = Rad::from(Deg(-90.0));
        let pitch = Rad(0.0);
        let fovy = Deg(90.0);

        let right = front.cross(world_up).normalize();

        Self {
            position,
            front,
            up,
            right,
            world_up,
            yaw,
            pitch,
            fovy,
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    left: f32,
    right: f32,
    forward: f32,
    backward: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}
impl CameraController {
    fn handle_keyboard(&mut self, key: KeyCode, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            KeyCode::KeyW => {
                self.forward = amount;
            }
            KeyCode::KeyS => {
                self.backward = amount;
            }
            KeyCode::KeyA => {
                self.left = amount;
            }
            KeyCode::KeyD => {
                self.right = amount;
            }
            _ => {}
        }
    }

    fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        self.rotate_horizontal = dx as f32;
        self.rotate_vertical = dy as f32;
    }

    fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        camera.position += camera.front * (self.forward - self.backward) * self.speed * dt;
        camera.position += camera.right * (self.right - self.left) * self.speed * dt;

        let new_yaw = camera.yaw + Rad(self.rotate_horizontal) * self.sensitivity * dt;
        let new_pitch = camera.pitch - Rad(self.rotate_vertical) * self.sensitivity * dt;
        camera.yaw = new_yaw % Rad(PI * 2.0);
        camera.pitch = clamp(new_pitch, -Rad(FRAC_PI_2 - 0.0001), Rad(FRAC_PI_2 - 0.0001));
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        let (yaw_s, yaw_c) = camera.yaw.sin_cos();
        let (pitch_s, pitch_c) = camera.pitch.sin_cos();
        camera.front = vec3(yaw_c * pitch_c, pitch_s, yaw_s * pitch_c).normalize();
        camera.right = camera.front.cross(camera.world_up).normalize();
        camera.up = camera.right.cross(camera.front).normalize();
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            left: Default::default(),
            right: Default::default(),
            forward: Default::default(),
            backward: Default::default(),
            rotate_horizontal: Default::default(),
            rotate_vertical: Default::default(),
            speed: 2.5,
            sensitivity: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct CameraWrapper {
    pub camera: Camera,
    camera_controller: CameraController,
}

impl Default for CameraWrapper {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            camera_controller: Default::default(),
        }
    }
}
impl CameraWrapper {
    pub fn update(&mut self, dt: Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
    }

    pub fn handle_mouse_motion(&mut self, dx: f64, dy: f64) {
        self.camera_controller.handle_mouse_motion(dx, dy);
    }
    pub fn handle_keyboard(&mut self, key: KeyCode, state: ElementState) {
        self.camera_controller.handle_keyboard(key, state);
    }
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        self.camera.get_view_matrix()
    }
    pub fn get_projection_matrix(&self, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
        self.camera.get_projection_matrix(aspect, znear, zfar)
    }

    pub fn get_camera_uniform_data(&self, aspect: f32, znear: f32, zfar: f32) -> CameraUniformData {
        let view = self.get_view_matrix().into();
        let proj = self.get_projection_matrix(aspect, znear, zfar).into();
        let pos = Point3::from_vec(self.camera.position)
            .to_homogeneous()
            .into();
        CameraUniformData { view, proj, pos }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniformData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub pos: [f32; 4],
}
