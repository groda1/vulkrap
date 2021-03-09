use crate::engine::entity::{Entity, EntityHandle};
use crate::renderer::context::{Context, PipelineRenderJob, RenderJob};

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    entities: Vec<Entity>,

    // Static terrain system. Quadtree with static terrain +  entity links?
    render_job_buffer: RenderJob,
}

impl Scene {
    pub fn new() -> Scene {
        let mut render_job_buffer = Vec::with_capacity(10);
        render_job_buffer.push((0, Vec::with_capacity(100)));

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

    pub fn get_render_job(&mut self) -> &RenderJob {
        self.render_job_buffer[0].1.clear();

        for entity in self.entities.iter() {
            self.render_job_buffer[0].1.push(PipelineRenderJob::new(
                entity.vertex_buffer,
                entity.index_buffer,
                entity.indices.len() as u32,
            ));
        }

        &self.render_job_buffer
    }
}
