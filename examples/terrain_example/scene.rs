use cgmath::{Deg, Quaternion, Rotation3};
use vulkrap::engine::datatypes::ModelWoblyPushConstant;

use vulkrap::engine::entity::WobblyEntity;
use vulkrap::engine::mesh::MeshManager;
use vulkrap::engine::terrain::Terrain;
use vulkrap::renderer::context::Context;
use vulkrap::renderer::rawarray::RawArrayPtr;
use vulkrap::renderer::types::{DrawCommand, PipelineHandle};

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    wobbly_objects: Vec<WobblyEntity>,
    wobbly_pipeline: PipelineHandle,

    terrain: Terrain,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        _mesh_manager: &MeshManager,
        wobbly_pipeline: PipelineHandle,
        terrain_pipeline: PipelineHandle,
    ) -> Scene {





        Scene {
            wobbly_objects: vec![],
            wobbly_pipeline,
            terrain: Terrain::new(context, terrain_pipeline),
        }
    }

    pub fn add_wobbly_entity(&mut self, entity: WobblyEntity) {
        self.wobbly_objects.push(entity);
    }

    pub fn update(&mut self, delta_time_s: f32) {
        const ROT_SPEED: f32 = 25.0;
        for entity in self.wobbly_objects.iter_mut() {
            entity.orientation = entity.orientation * Quaternion::from_angle_z(Deg(-delta_time_s * ROT_SPEED));
            entity.wobble += delta_time_s * 5.0;

            entity.update_push_constant_buffer();
        }
    }

    pub fn draw(&mut self, context: &mut Context) {
        for entity in self.wobbly_objects.iter() {
            context.add_draw_command(DrawCommand::new_buffered(
                self.wobbly_pipeline,
                &entity.push_constant_buf as *const ModelWoblyPushConstant as RawArrayPtr,
                entity.mesh.vertex_data,
                1,
                0,
            ));
        }

        self.terrain.draw(context);
    }

}
