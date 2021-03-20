use cgmath::{Deg, Quaternion, Rotation3};

use crate::engine::datatypes::{ModelColorPushConstant, ModelWoblyPushConstant};
use crate::engine::entity::{FlatColorEntity, WobblyEntity};
use crate::engine::terrain::Terrain;
use crate::renderer::context::Context;
use crate::renderer::pipeline::{PipelineDrawCommand, PipelineHandle, PipelineJob};

const WOBBLY_INDEX: usize = 0;
const FLAT_COLOR_INDEX: usize = 1;
const TERRAIN_INDEX: usize = 2;

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    wobbly_objects: Vec<WobblyEntity>,
    flat_objects: Vec<FlatColorEntity>,

    terrain: Terrain,
    render_job_buffer: Vec<PipelineJob>,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        static_objects_pipeline: PipelineHandle,
        flat_objects_pipeline: PipelineHandle,
        terrain_pipeline: PipelineHandle,
    ) -> Scene {
        let mut render_job_buffer = Vec::new();

        render_job_buffer.push(PipelineJob::new(static_objects_pipeline));
        render_job_buffer.push(PipelineJob::new(flat_objects_pipeline));
        render_job_buffer.push(PipelineJob::new(terrain_pipeline));

        Scene {
            wobbly_objects: Vec::new(),
            flat_objects: Vec::new(),
            render_job_buffer,
            terrain: Terrain::new(context),
        }
    }

    pub fn add_wobbly_entity(&mut self, entity: WobblyEntity) {
        self.wobbly_objects.push(entity);
    }

    pub fn add_flat_color_entity(&mut self, entity: FlatColorEntity) {
        self.flat_objects.push(entity);
    }

    pub fn update(&mut self, delta_time_s: f32) {
        const ROT_SPEED: f32 = 25.0;
        for entity in self.wobbly_objects.iter_mut() {
            entity.orientation = entity.orientation * Quaternion::from_angle_z(Deg(-delta_time_s * ROT_SPEED));
            entity.wobble += delta_time_s * 5.0;

            entity.update_push_constant_buffer();
        }
    }

    pub fn get_render_job(&mut self) -> &Vec<PipelineJob> {
        self.render_job_buffer[WOBBLY_INDEX].draw_commands.clear();
        self.render_job_buffer[FLAT_COLOR_INDEX].draw_commands.clear();

        for entity in self.wobbly_objects.iter() {
            self.render_job_buffer[WOBBLY_INDEX]
                .draw_commands
                .push(PipelineDrawCommand::new(
                    entity.mesh.vertex_buffer,
                    entity.mesh.index_buffer,
                    entity.mesh.index_count,
                    &entity.push_constant_buf as *const ModelWoblyPushConstant as *const u8,
                ));
        }

        for entity in self.flat_objects.iter() {
            self.render_job_buffer[FLAT_COLOR_INDEX]
                .draw_commands
                .push(PipelineDrawCommand::new(
                    entity.mesh.vertex_buffer,
                    entity.mesh.index_buffer,
                    entity.mesh.index_count,
                    &entity.push_constant_buf as *const ModelColorPushConstant as *const u8,
                ));
        }

        self.terrain
            .set_draw_commands(&mut self.render_job_buffer[TERRAIN_INDEX].draw_commands);

        &self.render_job_buffer
    }
}
