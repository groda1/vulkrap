use std::f32::consts::{FRAC_PI_2, PI};
use cgmath::{Vector2, Vector3};
use rotate_enum::RotateEnum;

use vulkrap::engine::camera::Camera;
use vulkrap::renderer::context::Context;

const CAMERA_HEIGHT: f32 = 0.4;
const MAX_BUFFERED_MOVES: usize = 3;

#[derive(Copy, Clone, RotateEnum)]
pub enum MovementInput {
    Forward,
    Right,
    Backward,
    Left,
}

pub enum RotationalInput {
    Left,
    Right
}

#[derive(Copy, Clone, RotateEnum)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

pub struct Movement {
    is_moving: bool,
    movement_input: Vec<MovementInput>,
    discrete_position: Vector2<i32>,
    real_position: Vector2<f32>,
    orientation: Orientation,
}

impl Movement {
    pub fn new(position: Vector2<i32>, orientation: Orientation) -> Movement {
        Movement {
            is_moving: false,
            movement_input: Vec::with_capacity(MAX_BUFFERED_MOVES),
            discrete_position: position,
            real_position: position.map(|p| p as f32),
            orientation,
        }
    }

    pub fn add_move_input(&mut self, input: MovementInput) {
        let adjusted_input = match self.orientation {
            Orientation::North => { input }
            Orientation::East => {input.next() }
            Orientation::South => {input.next().next() }
            Orientation::West => {input.prev() }
        };

        let movement = match adjusted_input {
            MovementInput::Forward => { Vector2::new(0, -1)}
            MovementInput::Right => { Vector2::new(1, 0)}
            MovementInput::Backward => { Vector2::new(0, 1)}
            MovementInput::Left => { Vector2::new(-1, 0)}
        };

        self.discrete_position += movement;
        self.real_position = Vector2::new(self.discrete_position.x as f32, self.discrete_position.y as f32);
    }

    pub fn add_rot_input(&mut self, input: RotationalInput) {
        match input {
            RotationalInput::Left => { self.orientation = self.orientation.prev() }
            RotationalInput::Right => {self.orientation = self.orientation.next() }
        }
    }

    pub fn update_camera(&self, context: &mut Context, camera: &mut Camera) {
        let yaw = match self.orientation {
            Orientation::North => { 0.0 }
            Orientation::East => { -FRAC_PI_2 }
            Orientation::South => { PI }
            Orientation::West => { FRAC_PI_2 }
        };

        camera.set_yaw(yaw);
        camera.set_position(Vector3::new(self.real_position.x, CAMERA_HEIGHT, self.real_position.y));


        camera.update_uniform(context);
    }
}