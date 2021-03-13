use crate::engine::datatypes::{ColoredVertex, Index};
use crate::engine::mesh::Mesh;
use ash::vk;
use cgmath::{Matrix4, One, Quaternion, SquareMatrix, Vector3, Zero};

pub type EntityHandle = usize;

pub struct Entity {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub mesh: Mesh,
}

impl Entity {
    pub fn new(position: Vector3<f32>, orientation: Quaternion<f32>, mesh: Mesh) -> Entity {
        Entity {
            position: Vector3::zero(),
            orientation: Quaternion::one(),
            mesh,
        }
    }
}
