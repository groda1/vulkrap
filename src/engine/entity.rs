use cgmath::{Quaternion, Vector3};

use crate::engine::mesh::Mesh;
use std::ptr;
use crate::renderer::pipeline::PushConstantData;

pub type EntityHandle = usize;

pub struct WobblyEntity {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    //pub scale: f32,
    pub mesh: Mesh,
    pub wobble: f32
}

impl WobblyEntity {
    pub fn new(position: Vector3<f32>, orientation: Quaternion<f32>, mesh: Mesh, wobble : f32) -> WobblyEntity {
        WobblyEntity {
            position,
            orientation,
            mesh,
            wobble
        }
    }
}
