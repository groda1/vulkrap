use cgmath::{Deg, Matrix4, Quaternion, Rotation3, Zero, One, Matrix};

use crate::engine::entity::{WobblyEntity, EntityHandle};
use crate::renderer::pipeline::{PipelineDrawCommand, PipelineHandle, PushConstantData, PipelineJob};
use std::ptr;

const STATIC_OBJECTS_INDEX: usize = 0;

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    wobbly_objects: Vec<WobblyEntity>,
    // Static terrain system. Quadtree with static terrain +  entity links?


    render_job_buffer: Vec<PipelineJob<ModelWoblyPushConstant>>,
}

impl Scene {
    pub fn new(static_objects_pipeline: PipelineHandle) -> Scene {
        let mut render_job_buffer = Vec::with_capacity(10);

        render_job_buffer.push(PipelineJob::new(static_objects_pipeline));

        Scene {
            wobbly_objects: Vec::with_capacity(100),
            render_job_buffer,

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

        }
    }

    pub fn get_render_job(&mut self) -> &Vec<PipelineJob<ModelWoblyPushConstant>> {
        self.render_job_buffer[STATIC_OBJECTS_INDEX].draw_commands.clear();

        for entity in self.wobbly_objects.iter() {
            let transform = Matrix4::from_translation(entity.position) * Matrix4::from(entity.orientation);

            self.render_job_buffer[STATIC_OBJECTS_INDEX]
                .draw_commands
                .push(PipelineDrawCommand::new(
                    entity.mesh.vertex_buffer,
                    entity.mesh.index_buffer,
                    entity.mesh.index_count,
                    ModelWoblyPushConstant::new(transform, entity.wobble)
                ));
        }

        &self.render_job_buffer
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ModelWoblyPushConstant {
    data: [u8; 68]
}
impl ModelWoblyPushConstant {
    pub fn new(model_transform: Matrix4<f32>, wobble: f32) -> ModelWoblyPushConstant {
        let mut data = [0 as u8; 68];
        unsafe {
            ptr::copy_nonoverlapping(model_transform.as_ptr() as *const u8 , data.as_mut_ptr(), 64);
            ptr::copy_nonoverlapping(&wobble as *const f32 as *const u8, data.as_mut_ptr().offset(64), 4);
        }

        ModelWoblyPushConstant {
            data
        }
    }
}

impl PushConstantData for ModelWoblyPushConstant {
    fn get_bytes(&self) -> &[u8] {
        &self.data
    }
}
