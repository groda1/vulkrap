use std::collections::VecDeque;
use std::f32::consts::{FRAC_PI_2, PI};
use cgmath::{Vector2, Vector3, VectorSpace};
use rotate_enum::RotateEnum;

use vulkrap::engine::camera::Camera;
use vulkrap::engine::math::lerp;
use vulkrap::renderer::context::Context;

const CAMERA_HEIGHT: f32 = 0.4;
const MAX_BUFFERED_MOVES: usize = 3;

const MOVE_SPEED: f32 = 4.0;


#[derive(Copy, Clone)]
pub enum MovementInput {
    Forward,
    Right,
    Backward,
    Left,
    RotateLeft,
    RotateRight,
}

impl MovementInput {
    fn is_rotation(&self) -> bool {
        self.rotation().is_some()
    }

    fn is_translation(&self) -> bool {
        self.translation().is_some()
    }

    fn translation(&self) -> Option<TranslationInput> {
        match self {
            MovementInput::Forward => { Some(TranslationInput::Forward) }
            MovementInput::Right => { Some(TranslationInput::Right) }
            MovementInput::Backward => { Some(TranslationInput::Backward) }
            MovementInput::Left => { Some(TranslationInput::Left) }
            _ => { None }
        }
    }

    fn rotation(&self) -> Option<RotationalInput> {
        match self {
            MovementInput::RotateLeft => { Some(RotationalInput::Left) }
            MovementInput::RotateRight => { Some(RotationalInput::Right) }
            _ => { None }
        }
    }
}

#[derive(Copy, Clone, RotateEnum)]
enum TranslationInput {
    Forward,
    Right,
    Backward,
    Left,
}


#[derive(Copy, Clone)]
enum RotationalInput {
    Left,
    Right,
}

#[derive(Copy, Clone, RotateEnum)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    fn to_yaw(&self) -> f32 {
        match self {
            Orientation::North => { 0.0 }
            Orientation::East => { -FRAC_PI_2 }
            Orientation::South => { PI }
            Orientation::West => { FRAC_PI_2 }
        }
    }
}

pub struct Movement {
    current_move: Option<MovementInput>,
    current_move_progress: f32,
    movement_input: VecDeque<MovementInput>,

    from_discrete_position: Vector2<i32>,
    pub discrete_position: Vector2<i32>,
    real_position: Vector2<f32>,

    to_yaw: f32,
    from_yaw: f32,
    current_yaw: f32,
    pub orientation: Orientation,
}

impl Movement {
    pub fn new(position: Vector2<i32>, orientation: Orientation) -> Movement {
        Movement {
            current_move: None,
            current_move_progress: 0.0,
            movement_input: VecDeque::with_capacity(MAX_BUFFERED_MOVES),
            from_discrete_position: position,
            discrete_position: position,
            real_position: position.map(|p| p as f32),
            to_yaw: 0.0,
            from_yaw: 0.0,
            current_yaw: orientation.to_yaw(),
            orientation,
        }
    }

    pub fn add_input(&mut self, input: MovementInput) {
        if self.movement_input.len() <= MAX_BUFFERED_MOVES {
            self.movement_input.push_back(input);
        }
    }


    pub fn update(&mut self, delta_time_s: f32) {
        if let Some(current_movement) = self.current_move {
            self.current_move_progress += MOVE_SPEED * delta_time_s;
            if self.current_move_progress >= 1.0 {
                self.current_move_progress = 1.0;
                self.current_move = None;
                self.current_yaw = self.orientation.to_yaw();
            } else if current_movement.is_translation() {
                let from = Vector2::new(self.from_discrete_position.x as f32, self.from_discrete_position.y as f32);
                let to = Vector2::new(self.discrete_position.x as f32, self.discrete_position.y as f32);
                self.real_position = from.lerp(to, self.current_move_progress);
            } else if current_movement.is_rotation() {
                self.current_yaw = lerp(self.from_yaw, self.to_yaw, self.current_move_progress);
            }
        } else {
            if let Some(movement) = self.movement_input.pop_front() {
                self.current_move_progress = 0.0;

                if let Some(translation) = movement.translation() {
                    let adjusted_translation = match self.orientation {
                        Orientation::North => { translation }
                        Orientation::East => { translation.next() }
                        Orientation::South => { translation.next().next() }
                        Orientation::West => { translation.prev() }
                    };

                    let movement_vector = match adjusted_translation {
                        TranslationInput::Forward => { Vector2::new(0, -1) }
                        TranslationInput::Right => { Vector2::new(1, 0) }
                        TranslationInput::Backward => { Vector2::new(0, 1) }
                        TranslationInput::Left => { Vector2::new(-1, 0) }
                    };

                    self.from_discrete_position = self.discrete_position;
                    self.discrete_position += movement_vector;
                } else if let Some(rotation) = movement.rotation() {
                    self.from_yaw = self.current_yaw;
                    match rotation { 
                        RotationalInput::Left => {
                            self.to_yaw = self.current_yaw + FRAC_PI_2 + 0.02;
                            self.orientation = self.orientation.prev();
                        },
                        RotationalInput::Right => {
                            self.to_yaw = self.current_yaw - FRAC_PI_2;
                            self.orientation = self.orientation.next();
                        },
                    }
                } else {
                    unreachable!()
                }
                self.current_move = Some(movement);
            }
        }
    }

    pub fn update_camera(&self, context: &mut Context, camera: &mut Camera) {
        camera.set_yaw(self.current_yaw);
        camera.set_position(Vector3::new(self.real_position.x, CAMERA_HEIGHT, self.real_position.y));

        camera.update_uniform(context);
    }
}