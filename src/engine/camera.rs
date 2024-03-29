use crate::engine::cvars::{ConfigVariables, FOV, M_PITCH, M_SENSITIVITY, M_YAW};
use crate::engine::datatypes::ViewProjectionUniform;
use crate::renderer::context::Context;
use crate::renderer::types::BufferObjectHandle;
use crate::renderer::types::{UniformHandle, UniformStage};
use cgmath::{dot, Deg, Matrix4, Quaternion, Rad, Rotation3, Vector3};

const MOVE_SPEED: f32 = 25.0;

const YAW_LIMIT: f32 = std::f32::consts::PI * 2.0;
const PITCH_LIMIT: f32 = (std::f32::consts::PI / 2.0) - 0.05;

pub struct Camera {
    position: Vector3<f32>,

    pitch: f32,
    yaw: f32,

    uniform: BufferObjectHandle,

    sens_pitch: f32,
    sens_yaw: f32,
    sens_global: f32,

    fovy: f32,

    _flight_mode: bool,
}

impl Camera {
    pub fn new(context: &mut Context, config: &ConfigVariables) -> Self {
        let uniform = context.create_uniform_buffer::<ViewProjectionUniform>(UniformStage::Vertex);

        let mut cam = Camera {
            position: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            uniform,

            sens_pitch: 0.0,
            sens_yaw: 0.0,
            sens_global: 0.0,

            fovy: 60.0,

            _flight_mode: true,
        };
        cam.reconfigure(config);
        cam.update_uniform(context);
        cam
    }

    pub fn get_uniform(&self) -> UniformHandle {
        self.uniform
    }

    pub fn reconfigure(&mut self, config: &ConfigVariables) {
        self.sens_pitch = config.get(M_PITCH).as_float();
        self.sens_yaw = config.get(M_YAW).as_float();
        self.sens_global = config.get(M_SENSITIVITY).as_float();
        self.fovy = config.get(FOV).as_float();
    }

    pub fn update_uniform(&mut self, context: &mut Context) {
        let data = ViewProjectionUniform {
            view: self._get_view_matrix(),
            proj: cgmath::perspective(Deg(self.fovy), context.get_aspect_ratio(), 0.1, 1000.0),
        };
        context.set_buffer_object(self.uniform, data);
    }

    pub fn update_yaw_pitch(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw * (-self.sens_yaw) * self.sens_global;
        if self.yaw > YAW_LIMIT {
            self.yaw -= YAW_LIMIT;
        } else if self.yaw < -YAW_LIMIT {
            self.yaw += YAW_LIMIT;
        }

        self.pitch += delta_pitch * (-self.sens_pitch) * self.sens_global;
        if self.pitch > PITCH_LIMIT {
            self.pitch = PITCH_LIMIT;
        } else if self.pitch < -PITCH_LIMIT {
            self.pitch = -PITCH_LIMIT;
        }
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn move_(&mut self, direction: Vector3<f32>, delta_time_s: f32) {
        self.position += Quaternion::from_angle_y(Rad(self.yaw)) * (direction * MOVE_SPEED * delta_time_s);
    }

    fn _get_view_matrix(&self) -> Matrix4<f32> {
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        let xaxis = Vector3::new(cos_yaw, 0.0, -sin_yaw);
        let yaxis = Vector3::new(sin_yaw * sin_pitch, cos_pitch, cos_yaw * sin_pitch);
        let zaxis = Vector3::new(sin_yaw * cos_pitch, -sin_pitch, cos_pitch * cos_yaw);

        Matrix4::new(
            xaxis.x,
            yaxis.x,
            zaxis.x,
            0.0,
            xaxis.y,
            yaxis.y,
            zaxis.y,
            0.0,
            xaxis.z,
            yaxis.z,
            zaxis.z,
            0.0,
            -dot(xaxis, self.position),
            -dot(yaxis, self.position),
            -dot(zaxis, self.position),
            1.0,
        )
    }
}
