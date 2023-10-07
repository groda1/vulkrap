use std::path::Path;
use cgmath::{Deg, Matrix4, Vector3, Vector4};
use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::ConfigVariables;

use vulkrap::engine::datatypes::{Mesh, SimpleVertex};
use vulkrap::engine::mesh::MeshManager;
use vulkrap::engine::mesh::PredefinedMesh::SimpleQuad;
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{DrawCommand, PipelineConfiguration, PipelineHandle, SWAPCHAIN_PASS, VertexTopology};
use vulkrap::util::file;

const BLOCK_WIDTH:usize = 16;
const BLOCK_HEIGHT: usize = 16;
const BLOCK_SIZE: usize = BLOCK_WIDTH * BLOCK_HEIGHT;


#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct GeometryPushConstant {
    transform: Matrix4<f32>,
    color: Vector4<f32>,
}

impl GeometryPushConstant {
    pub fn new(transform: Matrix4<f32>, color: Vector4<f32>) -> Self {
        GeometryPushConstant { transform, color }
    }
}

#[derive(Clone, Debug, Copy)]
struct Geometry {
    mesh: Mesh,
    push_constant: GeometryPushConstant,
}


struct Block {
    grid: [Option<Cell>; BLOCK_SIZE]
}

impl Block {

    pub fn new() -> Block {
        Block {
            grid: [None; BLOCK_SIZE]
        }
    }

    pub fn set(&mut self, x: i32, y:i32, color: Vector4<f32>, mesh: Mesh) {

        let rot = Matrix4::from_angle_x(Deg(-90.0));
        let translation = Matrix4::from_translation(Vector3::new(x as f32, 0.0, y as f32));

        let cell = Cell::new(translation* rot  , color, mesh);

        self.grid[y as usize * BLOCK_WIDTH + x as usize] = Some(cell);
    }

    pub fn draw(&mut self, context: &mut Context, pipeline: PipelineHandle) {

        for i in self.grid.iter() {
            if let Some(block)  = i {
                // Draw floor
                context.add_draw_command(DrawCommand::new_buffered(pipeline, &block.floor.push_constant,
                    block.floor.mesh.vertex_data,
                ));
            }
        }
    }
}

#[derive(Clone, Debug, Copy)]
struct Cell {
    floor: Geometry
}

impl Cell {
    pub fn new(transform: Matrix4<f32>, color : Vector4<f32>, mesh: Mesh) -> Cell {
        Cell {
            floor: Geometry {
                mesh,
                push_constant: GeometryPushConstant::new(transform, color)
            }
        }
    }
}

pub struct Scene {
    block: Block,
    geometry_pipeline: PipelineHandle
}

impl Scene {


    pub fn new(context: &mut Context, mesh_manager: &MeshManager, camera: &Camera) -> Scene {


        let mesh = *mesh_manager.get_predefined_mesh(SimpleQuad);


        let pipeline_config = PipelineConfiguration::builder()
            .with_push_constant::<GeometryPushConstant>()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/flat_color_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/flat_color_frag.spv")))
            .with_vertex_topology(VertexTopology::Triangle)
            .with_vertex_uniform(0, camera.get_uniform())
            .build();

        let pipeline = context.add_pipeline::<SimpleVertex>(SWAPCHAIN_PASS, pipeline_config);

        let mut block = Block::new();

        block.set(1, 1, Vector4::new(1.0, 0.8, 0.8, 1.0), mesh);
        block.set(1, 2, Vector4::new(1.0, 0.8, 0.8, 1.0), mesh);
        block.set(2, 1, Vector4::new(0.8, 0.4, 0.4, 1.0), mesh);
        block.set(2, 2, Vector4::new(0.8, 0.4, 0.4, 1.0), mesh);
        block.set(2, 3, Vector4::new(0.8, 0.4, 0.4, 1.0), mesh);
        block.set(2, 4, Vector4::new(0.8, 0.4, 0.4, 1.0), mesh);
        block.set(2, 5, Vector4::new(0.1, 0.5, 0.1, 1.0), mesh);
        block.set(1, 5, Vector4::new(0.1, 0.5, 0.1, 1.0), mesh);
        block.set(2, 6, Vector4::new(0.1, 0.5, 0.1, 1.0), mesh);
        block.set(1, 6, Vector4::new(0.1, 0.5, 0.1, 1.0), mesh);


        Scene {
            block,
            geometry_pipeline: pipeline
        }

    }

    pub fn reconfigure(&mut self, config: &ConfigVariables) {

    }

    pub fn update(&mut self, context: &mut Context, delta_time_s: f32) {

    }

    pub fn draw(&mut self, context: &mut Context) {
        self.block.draw(context, self.geometry_pipeline);

    }


}