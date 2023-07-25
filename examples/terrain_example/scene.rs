use vulkrap::engine::mesh::MeshManager;
use vulkrap::engine::terrain::Terrain;
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::PipelineHandle;

pub struct Scene {
    // TODO replace with entity content system ( specs? )

    terrain: Terrain,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        _mesh_manager: &MeshManager,
        terrain_pipeline: PipelineHandle,
        terrain_pipeline2: PipelineHandle,
    ) -> Scene {


        Scene {
            terrain: Terrain::new(context, terrain_pipeline, terrain_pipeline2),
        }
    }

    pub fn update(&mut self, delta_time_s: f32) {
    }

    pub fn draw(&mut self, context: &mut Context) {

        self.terrain.draw(context);
    }

}
