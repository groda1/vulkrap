use cgmath::{Deg, Matrix4, Quaternion, Rotation3};

use crate::engine::entity::{Entity, EntityHandle};
use crate::renderer::pipeline::{PipelineDrawCommand, PipelineHandle, PipelineJob};

const STATIC_OBJECTS_INDEX: usize = 0;

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    static_objects: Vec<Entity>,

    // Static terrain system. Quadtree with static terrain +  entity links?
    render_job_buffer: Vec<PipelineJob>,
}

impl Scene {
    pub fn new(static_objects_pipeline: PipelineHandle) -> Scene {
        let mut render_job_buffer = Vec::with_capacity(10);

        render_job_buffer.push(PipelineJob::new(static_objects_pipeline));

        Scene {
            static_objects: Vec::with_capacity(100),
            render_job_buffer,
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> EntityHandle {
        // This sucks!
        let handle = self.static_objects.len();
        self.static_objects.push(entity);

        handle
    }

    pub fn update(&mut self, delta_time_s: f32) {
        const ROT_SPEED: f32 = 25.0;
        for entity in self.static_objects.iter_mut() {
            entity.orientation = entity.orientation * Quaternion::from_angle_z(Deg(delta_time_s * ROT_SPEED));
        }
    }

    pub fn get_render_job(&mut self) -> &Vec<PipelineJob> {
        self.render_job_buffer[STATIC_OBJECTS_INDEX].draw_commands.clear();

        for entity in self.static_objects.iter() {
            let transform = Matrix4::from_translation(entity.position) * Matrix4::from(entity.orientation);

            self.render_job_buffer[STATIC_OBJECTS_INDEX]
                .draw_commands
                .push(PipelineDrawCommand::new(
                    entity.mesh.vertex_buffer,
                    entity.mesh.index_buffer,
                    entity.mesh.index_count,
                    transform,
                ));
        }

        &self.render_job_buffer
    }
}
