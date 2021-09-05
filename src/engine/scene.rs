use cgmath::{Deg, Quaternion, Rotation3};

use crate::engine::datatypes::{ModelColorPushConstant, ModelWoblyPushConstant};
use crate::engine::entity::{FlatColorEntity, WobblyEntity};
use crate::engine::mesh::{MeshManager, PredefinedMesh};
use crate::engine::terrain::Terrain;
use crate::engine::ui::hud;
use crate::engine::ui::hud::HUD;
use crate::renderer::context::{Context, PipelineHandle};
use crate::renderer::pipeline::PipelineDrawCommand;

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    wobbly_objects: Vec<WobblyEntity>,
    flat_objects: Vec<FlatColorEntity>,

    wobbly_pipeline: PipelineHandle,
    flat_objects_pipeline: PipelineHandle,

    terrain: Terrain,
    hud: HUD,
    render_job_buffer: Vec<PipelineDrawCommand>,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        mesh_manager: &MeshManager,
        wobbly_pipeline: PipelineHandle,
        flat_objects_pipeline: PipelineHandle,
        terrain_pipeline: PipelineHandle,
    ) -> Scene {
        let render_job_buffer = Vec::new();
        let (window_width, window_height) = context.get_framebuffer_extent();

        Scene {
            wobbly_objects: vec![],
            flat_objects: vec![],
            wobbly_pipeline,
            flat_objects_pipeline,
            render_job_buffer,
            terrain: Terrain::new(context, terrain_pipeline),
            hud: HUD::new(
                context,
                *mesh_manager.get_predefined_mesh(PredefinedMesh::TexturedQuad),
                window_width,
                window_height,
            ),
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

    pub fn fetch_render_job(&mut self) -> &Vec<PipelineDrawCommand> {
        self.render_job_buffer.clear();

        for entity in self.wobbly_objects.iter() {
            self.render_job_buffer.push(PipelineDrawCommand::new(
                self.wobbly_pipeline,
                entity.mesh.vertex_buffer,
                entity.mesh.index_buffer,
                entity.mesh.index_count,
                &entity.push_constant_buf as *const ModelWoblyPushConstant as *const u8,
            ));
        }

        for entity in self.flat_objects.iter() {
            self.render_job_buffer.push(PipelineDrawCommand::new(
                self.flat_objects_pipeline,
                entity.mesh.vertex_buffer,
                entity.mesh.index_buffer,
                entity.mesh.index_count,
                &entity.push_constant_buf as *const ModelColorPushConstant as *const u8,
            ));
        }

        self.terrain.draw(&mut self.render_job_buffer);
        self.hud.draw(&mut self.render_job_buffer);

        &self.render_job_buffer
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, width: u32, height: u32) {
        self.hud.handle_window_resize(context, width, height);
    }
}
