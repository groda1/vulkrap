use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3, Vector4};
use num::Zero;

use crate::engine::datatypes::{Mesh, ModelColorPushConstant, ModelWoblyPushConstant};
use crate::renderer::types::VertexData;


#[derive(Debug)]
pub struct DefaultEntity {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub mesh: VertexData,
    pub color: Vector4<f32>,

    pub push_constant_buf: ModelColorPushConstant,
}

impl DefaultEntity {
     pub fn new(
         position: Vector3<f32>,
         scale: Vector3<f32>,
         orientation: Quaternion<f32>,
         mesh: VertexData,
         color: Vector4<f32>,
     ) -> Self {
         let mut entity = DefaultEntity {
             position,
             scale,
             orientation,
             mesh,
             color,
             push_constant_buf: ModelColorPushConstant::new(Matrix4::identity(), Vector4::zero()),
         };
         entity.update_push_constant_buffer();

         entity
     }
     pub fn update_push_constant_buffer(&mut self) {
         self.push_constant_buf = ModelColorPushConstant::new(
             Matrix4::from_translation(self.position)
                 * Matrix4::from(self.orientation)
                 * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z),
             self.color,
         );
     }
 }

#[derive(Debug)]
pub struct WobblyEntity {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub mesh: Mesh,
    pub wobble: f32,

    pub push_constant_buf: ModelWoblyPushConstant,
}

impl WobblyEntity {
    pub fn new(position: Vector3<f32>, orientation: Quaternion<f32>, mesh: Mesh, wobble: f32) -> Self {
        let mut entity = WobblyEntity {
            position,
            orientation,
            mesh,
            wobble,
            push_constant_buf: ModelWoblyPushConstant::default(),
        };

        entity.update_push_constant_buffer();

        entity
    }

    pub fn update_push_constant_buffer(&mut self) {
        self.push_constant_buf = ModelWoblyPushConstant::new(
            Matrix4::from_translation(self.position) * Matrix4::from(self.orientation),
            self.wobble,
        );
    }
}
