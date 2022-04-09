use cgmath::{Deg, Quaternion, Rotation3};

use crate::engine::datatypes::{ModelWoblyPushConstant, WindowExtent};
use crate::engine::entity::WobblyEntity;
use crate::engine::mesh::MeshManager;
use crate::engine::terrain::Terrain;

use crate::engine::console::Console;
use crate::engine::ui::hud::Hud;
use crate::renderer::context::Context;
use crate::renderer::rawarray::RawArrayPtr;
use crate::renderer::types::{DrawCommand, PipelineHandle};

pub struct Scene {
    // TODO replace with entity content system ( specs? )
    wobbly_objects: Vec<WobblyEntity>,
    wobbly_pipeline: PipelineHandle,

    terrain: Terrain,
    hud: Hud,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        mesh_manager: &MeshManager,
        wobbly_pipeline: PipelineHandle,
        terrain_pipeline: PipelineHandle,
    ) -> Scene {
        let (window_width, window_height) = context.get_framebuffer_extent();
        let window_extent = WindowExtent::new(window_width, window_height);

        Scene {
            wobbly_objects: vec![],
            wobbly_pipeline,
            terrain: Terrain::new(context, terrain_pipeline),
            hud: Hud::new(context, window_extent, mesh_manager),
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

    pub fn draw(&mut self, context: &mut Context, console: &Console) {
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
        self.hud.draw(context, console);
    }

    pub fn handle_window_resize(&mut self, context: &mut Context, new_extent: WindowExtent) {
        self.hud.handle_window_resize(context, new_extent);
    }
}
