use vulkrap::engine::mesh::MeshManager;
use vulkrap::engine::terrain::Terrain;
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::PipelineHandle;

pub struct Scene {
    terrain: Terrain,
}

impl Scene {
    pub fn new(
        context: &mut Context,
        _mesh_manager: &MeshManager,
        terrain_pipeline: PipelineHandle,
    ) -> Scene {


        Scene {
            terrain: Terrain::new(context, terrain_pipeline),
        }
    }

    pub fn update(&mut self, _delta_time_s: f32) {
    }

    pub fn draw(&mut self, context: &mut Context) {

        self.terrain.draw(context);
    }

}
