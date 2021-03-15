use cgmath::{dot, Matrix4, Quaternion, Rad, Rotation3, Vector3};

const M_SENSITIVITY: f32 = 0.1;
const M_YAW: f32 = -0.01;
const M_PITCH: f32 = -0.01;

const MOVE_SPEED: f32 = 2.5;

const YAW_LIMIT: f32 = std::f32::consts::PI * 2.0;
const PITCH_LIMIT: f32 = std::f32::consts::PI * 2.0;

pub struct Camera {
    position: Vector3<f32>,
    pitch: f32,
    yaw: f32,

    _flight_mode: bool,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vector3::new(0.0, 0.0, 3.0),
            pitch: 0.0,
            yaw: 0.0,
            _flight_mode: true,
        }
    }

    pub fn _position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn move_forward(&mut self, delta_time_s: f32) {
        self.position += Quaternion::from_angle_y(Rad(self.yaw)) * Vector3::new(0.0, 0.0, delta_time_s * -MOVE_SPEED);
    }

    pub fn move_backward(&mut self, delta_time_s: f32) {
        self.position += Quaternion::from_angle_y(Rad(self.yaw)) * Vector3::new(0.0, 0.0, delta_time_s * MOVE_SPEED);
    }

    pub fn move_left(&mut self, delta_time_s: f32) {
        self.position += Quaternion::from_angle_y(Rad(self.yaw)) * Vector3::new(delta_time_s * -MOVE_SPEED, 0.0, 0.0);
    }

    pub fn move_right(&mut self, delta_time_s: f32) {
        self.position += Quaternion::from_angle_y(Rad(self.yaw)) * Vector3::new(delta_time_s * MOVE_SPEED, 0.0, 0.0);
    }

    pub fn debug_position(&self) {
        log_debug!("pos {:?}", self.position)
    }

    pub fn update_yaw_pitch(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw * M_YAW * M_SENSITIVITY;
        if self.yaw > YAW_LIMIT {
            self.yaw -= YAW_LIMIT;
        } else if self.yaw < -YAW_LIMIT {
            self.yaw += YAW_LIMIT;
        }

        self.pitch += delta_pitch * M_PITCH * M_SENSITIVITY;
        if self.pitch > PITCH_LIMIT {
            self.pitch -= PITCH_LIMIT;
        } else if self.pitch < -PITCH_LIMIT {
            self.pitch += PITCH_LIMIT;
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
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
