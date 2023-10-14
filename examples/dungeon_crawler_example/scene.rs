use std::path::Path;
use cgmath::{Deg, Matrix4, Vector3, Vector4};
use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::ConfigVariables;

use vulkrap::engine::datatypes::{Mesh, NormalVertex};
use vulkrap::engine::mesh::{MeshHandle, MeshManager};
use vulkrap::engine::mesh::PredefinedMesh::NormaledQuad;
use vulkrap::{log, log_debug};
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{DrawCommand, PipelineConfiguration, PipelineHandle, SWAPCHAIN_PASS, VertexTopology};
use vulkrap::util::file;

const BLOCK_WIDTH: usize = 16;
const BLOCK_HEIGHT: usize = 16;
const BLOCK_SIZE: usize = BLOCK_WIDTH * BLOCK_HEIGHT;


#[repr(C)]
#[derive(Debug)]
pub struct GeometryPushConstant {
    transform: Matrix4<f32>,
    color: Vector4<f32>,
}

impl GeometryPushConstant {
    pub fn new(transform: Matrix4<f32>, color: Vector4<f32>) -> Self {
        GeometryPushConstant { transform, color }
    }
}

#[derive(Debug)]
struct Geometry {
    mesh: Mesh,
    push_constant: GeometryPushConstant,
}


struct Block {
    grid: Box<[Option<Cell>; BLOCK_SIZE]>,
}

impl Block {
    pub fn new() -> Block {
        Block {
            grid: Box::new(std::array::from_fn(|_| None))
        }
    }

    pub fn set(&mut self, x: i32, y: i32, color: Vector4<f32>, mesh: Mesh) {
        let floor_rot = Matrix4::from_angle_x(Deg(-90.0));
        let floor_trans = Matrix4::from_translation(Vector3::new(x as f32, 0.0, y as f32));
        let floor_geom = Geometry {
            mesh,
            push_constant: GeometryPushConstant::new(floor_trans * floor_rot, color - Vector4::new(0.3, 0.3, 0.3, 0.0)),
        };

        let mut cell = Cell::new(floor_geom);

        let west_wall_rot = Matrix4::from_angle_y(Deg(90.0));
        let west_wall_trans = Matrix4::from_translation(Vector3::new(x as f32 - 0.5, 0.5, y as f32));
        let west_wall_geom = Geometry {
            mesh,
            push_constant: GeometryPushConstant::new(west_wall_trans * west_wall_rot, color),
        };
        let east_wall_rot = Matrix4::from_angle_y(Deg(-90.0));
        let east_wall_trans = Matrix4::from_translation(Vector3::new(x as f32 + 0.5, 0.5, y as f32));
        let east_wall_geom = Geometry {
            mesh,
            push_constant: GeometryPushConstant::new(east_wall_trans * east_wall_rot, color),
        };

        let north_wall_rot = Matrix4::from_angle_y(Deg(0.0));
        let north_wall_trans = Matrix4::from_translation(Vector3::new(x as f32, 0.5, y as f32 - 0.5));
        let north_wall_geom = Geometry {
            mesh,
            push_constant: GeometryPushConstant::new(north_wall_trans * north_wall_rot, color),
        };

        let south_wall_rot = Matrix4::from_angle_y(Deg(180.0));
        let south_wall_trans = Matrix4::from_translation(Vector3::new(x as f32, 0.5, y as f32 + 0.5));
        let south_wall_geom = Geometry {
            mesh,
            push_constant: GeometryPushConstant::new(south_wall_trans * south_wall_rot, color),
        };

        cell.west_wall = Some(west_wall_geom);
        cell.east_wall = Some(east_wall_geom);
        cell.north_wall = Some(north_wall_geom);
        cell.south_wall = Some(south_wall_geom);

        /* Clear walls */
        if x > 0 {
            if let Some(western_cell) = &mut self.grid[y as usize * BLOCK_WIDTH + (x - 1) as usize] {
                cell.west_wall = None;
                western_cell.east_wall = None;
            }
        }
        if (x + 1) < BLOCK_WIDTH as i32 {
            if let Some(eastern_cell) = &mut self.grid[y as usize * BLOCK_WIDTH + (x + 1) as usize] {
                cell.east_wall = None;
                eastern_cell.west_wall = None;
            }
        }
        if y > 0 {
            if let Some(northern_cell) = &mut self.grid[(y - 1) as usize * BLOCK_WIDTH + x as usize] {
                cell.north_wall = None;
                northern_cell.south_wall = None;
            }
        }
        if (y + 1) < BLOCK_HEIGHT as i32 {
            if let Some(southern_cell) = &mut self.grid[(y + 1) as usize * BLOCK_WIDTH + x as usize] {
                cell.south_wall = None;
                southern_cell.north_wall = None;
            }
        }

        self.grid[y as usize * BLOCK_WIDTH + x as usize] = Some(cell);
    }

    pub fn draw(&mut self, context: &mut Context, pipeline: PipelineHandle) {
        for i in self.grid.iter() {
            if let Some(block) = i {

                // Draw floor
                context.add_draw_command(DrawCommand::new_buffered(pipeline, &block.floor.push_constant,
                                                                   block.floor.mesh,
                ));

                if let Some(wall) = &block.west_wall {
                    context.add_draw_command(DrawCommand::new_buffered(pipeline, &wall.push_constant,
                                                                       wall.mesh,
                    ));
                }
                if let Some(wall) = &block.east_wall {
                    context.add_draw_command(DrawCommand::new_buffered(pipeline, &wall.push_constant,
                                                                       wall.mesh,
                    ));
                }
                if let Some(wall) = &block.north_wall {
                    context.add_draw_command(DrawCommand::new_buffered(pipeline, &wall.push_constant,
                                                                       wall.mesh,
                    ));
                }
                if let Some(wall) = &block.south_wall {
                    context.add_draw_command(DrawCommand::new_buffered(pipeline, &wall.push_constant,
                                                                       wall.mesh,
                    ));
                }
            }
        }
    }
}

#[derive(Debug)]
struct Cell {
    floor: Geometry,

    north_wall: Option<Geometry>,
    east_wall: Option<Geometry>,
    west_wall: Option<Geometry>,
    south_wall: Option<Geometry>,
}

impl Cell {
    pub fn new(floor: Geometry) -> Cell {
        Cell {
            floor,
            north_wall: None,
            east_wall: None,
            west_wall: None,
            south_wall: None,
        }
    }
}

pub struct Scene {
    block: Block,
    geometry_pipeline: PipelineHandle,
}

impl Scene {
    pub fn new(context: &mut Context, mesh_manager: &MeshManager, camera: &Camera) -> Scene {
        let mesh = *mesh_manager.get_mesh(NormaledQuad as MeshHandle);


        let pipeline_config = PipelineConfiguration::builder()
            .with_push_constant::<GeometryPushConstant>()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/dc_environ_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/dc_environ_frag.spv")))
            .with_vertex_topology(VertexTopology::Triangle)
            .with_vertex_uniform(0, camera.get_uniform())
            .build();

        let pipeline = context.add_pipeline::<NormalVertex>(SWAPCHAIN_PASS, pipeline_config);

        log_debug!("block size {}",   std::mem::size_of::<Block>() as u32);
        log_debug!("cell size {}",   std::mem::size_of::<Cell>() as u32);
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
        block.set(3, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(4, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(5, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(6, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(7, 2, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(7, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(7, 4, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(8, 2, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(8, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(8, 4, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(9, 2, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(9, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);
        block.set(9, 4, Vector4::new(0.8, 0.8, 0.8, 1.0), mesh);

        Scene {
            block,
            geometry_pipeline: pipeline,
        }
    }

    pub fn reconfigure(&mut self, config: &ConfigVariables) {}

    pub fn update(&mut self, context: &mut Context, delta_time_s: f32) {}

    pub fn draw(&mut self, context: &mut Context) {
        self.block.draw(context, self.geometry_pipeline);
    }
}