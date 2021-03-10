use crate::engine::entity::{Entity, EntityHandle};
use crate::renderer::context::Context;
use crate::renderer::pipeline::{PipelineDrawCommand, PipelineJob};

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    entities: Vec<Entity>,

    // Static terrain system. Quadtree with static terrain +  entity links?
    render_job_buffer: Vec<PipelineJob>,
}

impl Scene {
    pub fn new() -> Scene {
        let mut render_job_buffer = Vec::with_capacity(10);

        render_job_buffer.push(PipelineJob::new(0));

        Scene {
            entities: Vec::with_capacity(100),
            render_job_buffer,
        }
    }

    pub fn add_entity(&mut self, context: &mut Context, mut entity: Entity) -> EntityHandle {
        // This sucks!
        let handle = self.entities.len();

        entity.vertex_buffer = context.allocate_vertex_buffer(&entity.vertices);
        entity.index_buffer = context.allocate_index_buffer(&entity.indices);

        self.entities.push(entity);

        handle
    }

    pub fn get_render_job(&mut self) -> &Vec<PipelineJob> {
        // TODO This sucks!

        self.render_job_buffer[0].draw_commands.clear();

        for entity in self.entities.iter() {
            self.render_job_buffer[0].draw_commands.push(PipelineDrawCommand::new(
                entity.vertex_buffer,
                entity.index_buffer,
                entity.indices.len() as u32,
            ));
        }

        &self.render_job_buffer
    }
}
