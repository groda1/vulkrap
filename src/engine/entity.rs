use cgmath::{Quaternion, Vector3};

use crate::engine::mesh::Mesh;

pub type EntityHandle = usize;

pub struct Entity {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    //pub scale: f32,
    pub mesh: Mesh,
}

impl Entity {
    pub fn new(position: Vector3<f32>, orientation: Quaternion<f32>, mesh: Mesh) -> Entity {
        Entity {
            position,
            orientation,
            mesh,
        }
    }
}
