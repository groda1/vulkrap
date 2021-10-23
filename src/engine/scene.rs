use cgmath::{Deg, Quaternion, Rotation3};

use crate::engine::datatypes::ModelWoblyPushConstant;
use crate::engine::entity::WobblyEntity;
use crate::engine::mesh::MeshManager;
use crate::engine::terrain::Terrain;

use crate::engine::console::Console;
use crate::engine::ui::hud::HUD;
use crate::renderer::context::{Context, PipelineHandle, PushConstantBufHandler};
use crate::renderer::pipeline::PipelineDrawCommand;
use crate::renderer::pushconstants::PushConstantPtr;

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    wobbly_objects: Vec<WobblyEntity>,
    wobbly_pipeline: PipelineHandle,

    terrain: Terrain,
    hud: HUD,
    render_job_buffer: Vec<PipelineDrawCommand>,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        mesh_manager: &MeshManager,
        wobbly_pipeline: PipelineHandle,
        terrain_pipeline: PipelineHandle,
    ) -> Scene {
        let render_job_buffer = Vec::new();
        let (window_width, window_height) = context.get_framebuffer_extent();

        Scene {
            wobbly_objects: vec![],
            wobbly_pipeline,
            render_job_buffer,
            terrain: Terrain::new(context, terrain_pipeline),
            hud: HUD::new(context, mesh_manager, window_width, window_height),
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

    pub fn build_render_job(
        &mut self,
        context: &mut dyn PushConstantBufHandler,
        console: &Console,
    ) -> &Vec<PipelineDrawCommand> {
        self.render_job_buffer.clear();

        for entity in self.wobbly_objects.iter() {
            self.render_job_buffer.push(PipelineDrawCommand::new(
                self.wobbly_pipeline,
                &entity.push_constant_buf as *const ModelWoblyPushConstant as PushConstantPtr,
                entity.mesh.vertex_buffer,
                entity.mesh.index_buffer,
                entity.mesh.index_count,
            ));
        }

        self.terrain.draw(&mut self.render_job_buffer);
        self.hud.draw(context, &mut self.render_job_buffer, console);

        &self.render_job_buffer
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, width: u32, height: u32) {
        self.hud.handle_window_resize(context, width, height);
    }
}
