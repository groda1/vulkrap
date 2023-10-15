use std::path::Path;
use cgmath::{Deg, Matrix4, Vector3, Vector4};
use vulkrap::engine::camera::Camera;
use vulkrap::engine::cvars::ConfigVariables;

use vulkrap::engine::datatypes::{Mesh, NormalVertex, TransformColorPushConstant};
use vulkrap::engine::mesh::{MeshHandle, MeshManager};
use vulkrap::engine::mesh::PredefinedMesh::NormaledQuad;
use vulkrap::log_debug;
use vulkrap::renderer::context::Context;
use vulkrap::renderer::types::{DrawCommand, PipelineConfiguration, PipelineHandle, TextureHandle, VertexTopology};
use vulkrap::util::file;

const BLOCK_WIDTH: usize = 16;
const BLOCK_HEIGHT: usize = 16;
const BLOCK_SIZE: usize = BLOCK_WIDTH * BLOCK_HEIGHT;



#[derive(Debug)]
struct Geometry {
    mesh: Mesh,
    push_constant: TransformColorPushConstant,
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

    pub fn set(&mut self, x: i32, y: i32, color: Vector4<f32>, floor_mesh: Mesh, wall_mesh: Mesh) {
        let floor_rot = Matrix4::from_angle_x(Deg(-90.0));
        let floor_trans = Matrix4::from_translation(Vector3::new(x as f32, 0.0, y as f32));
        let floor_geom = Geometry {
            mesh: floor_mesh,
            push_constant: TransformColorPushConstant::new(floor_trans * floor_rot, color - Vector4::new(0.3, 0.3, 0.3, 0.0)),
        };
        let roof_rot = Matrix4::from_angle_x(Deg(90.0));
        let roof_trans = Matrix4::from_translation(Vector3::new(x as f32, 1.0, y as f32));
        let roof_geom = Geometry {
            mesh: floor_mesh,
            push_constant: TransformColorPushConstant::new(roof_trans * roof_rot, color - Vector4::new(0.3, 0.3, 0.3, 0.0)),
        };


        let mut cell = Cell::new(floor_geom, roof_geom);

        let west_wall_rot = Matrix4::from_angle_y(Deg(90.0));
        let west_wall_trans = Matrix4::from_translation(Vector3::new(x as f32 - 0.5, 0.5, y as f32));
        let west_wall_geom = Geometry {
            mesh: wall_mesh,
            push_constant: TransformColorPushConstant::new(west_wall_trans * west_wall_rot, color),
        };
        let east_wall_rot = Matrix4::from_angle_y(Deg(-90.0));
        let east_wall_trans = Matrix4::from_translation(Vector3::new(x as f32 + 0.5, 0.5, y as f32));
        let east_wall_geom = Geometry {
            mesh: wall_mesh,
            push_constant: TransformColorPushConstant::new(east_wall_trans * east_wall_rot, color),
        };

        let north_wall_rot = Matrix4::from_angle_y(Deg(0.0));
        let north_wall_trans = Matrix4::from_translation(Vector3::new(x as f32, 0.5, y as f32 - 0.5));
        let north_wall_geom = Geometry {
            mesh: wall_mesh,
            push_constant: TransformColorPushConstant::new(north_wall_trans * north_wall_rot, color),
        };

        let south_wall_rot = Matrix4::from_angle_y(Deg(180.0));
        let south_wall_trans = Matrix4::from_translation(Vector3::new(x as f32, 0.5, y as f32 + 0.5));
        let south_wall_geom = Geometry {
            mesh: wall_mesh,
            push_constant: TransformColorPushConstant::new(south_wall_trans * south_wall_rot, color),
        };

        cell.west_wall = Some(Box::new(west_wall_geom));
        cell.east_wall = Some(Box::new(east_wall_geom));
        cell.north_wall = Some(Box::new(north_wall_geom));
        cell.south_wall = Some(Box::new(south_wall_geom));

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
                // Draw roof
                context.add_draw_command(DrawCommand::new_buffered(pipeline, &block.roof.push_constant,
                                                                   block.roof.mesh,
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
    floor: Box<Geometry>,
    roof: Box<Geometry>,

    north_wall: Option<Box<Geometry>>,
    east_wall: Option<Box<Geometry>>,
    west_wall: Option<Box<Geometry>>,
    south_wall: Option<Box<Geometry>>,
}

impl Cell {
    pub fn new(floor: Geometry, roof: Geometry) -> Cell {
        Cell {
            floor: Box::new(floor),
            roof: Box::new(roof),
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
    target_texture_handle: TextureHandle,
}

impl Scene {
    pub fn new(context: &mut Context, mesh_manager: &mut MeshManager, camera: &Camera) -> Scene {
        let floor_mesh = *mesh_manager.get_mesh(NormaledQuad as MeshHandle);
        let (_, mesh) = mesh_manager.load_new_mesh(context, Path::new("./resources/models/wall.obj")).unwrap();
        let wall_mesh = *mesh;

        let render_texture = context.add_render_texture(384, 216);
        let pass = context.create_render_pass(render_texture, 1000).unwrap();

        let pipeline_config = PipelineConfiguration::builder()
            .with_push_constant::<TransformColorPushConstant>()
            .with_vertex_shader(file::read_file(Path::new("./resources/shaders/dc_environ_vert.spv")))
            .with_fragment_shader(file::read_file(Path::new("./resources/shaders/dc_environ_frag.spv")))
            .with_vertex_topology(VertexTopology::Triangle)
            .with_vertex_uniform(0, camera.get_uniform())
            .build();

        let pipeline = context.add_pipeline::<NormalVertex>(pass, pipeline_config);

        log_debug!("block size {}",   std::mem::size_of::<Block>() as u32);
        log_debug!("cell size {}",   std::mem::size_of::<Cell>() as u32);
        let mut block = Block::new();

        block.set(1, 1, Vector4::new(1.0, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(1, 2, Vector4::new(1.0, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(2, 1, Vector4::new(0.8, 0.4, 0.4, 1.0), floor_mesh, wall_mesh);
        block.set(2, 2, Vector4::new(0.8, 0.4, 0.4, 1.0), floor_mesh, wall_mesh);
        block.set(2, 3, Vector4::new(0.8, 0.4, 0.4, 1.0), floor_mesh, wall_mesh);
        block.set(2, 4, Vector4::new(0.8, 0.4, 0.4, 1.0), floor_mesh, wall_mesh);
        block.set(2, 5, Vector4::new(0.1, 0.5, 0.1, 1.0), floor_mesh, wall_mesh);
        block.set(1, 5, Vector4::new(0.1, 0.5, 0.1, 1.0), floor_mesh, wall_mesh);
        block.set(2, 6, Vector4::new(0.1, 0.5, 0.1, 1.0), floor_mesh, wall_mesh);
        block.set(1, 6, Vector4::new(0.1, 0.5, 0.1, 1.0), floor_mesh, wall_mesh);
        block.set(3, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(4, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(5, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(6, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(7, 2, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(7, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(7, 4, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(8, 2, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(8, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(8, 4, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(9, 2, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(9, 3, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);
        block.set(9, 4, Vector4::new(0.8, 0.8, 0.8, 1.0), floor_mesh, wall_mesh);

        Scene {
            block,
            geometry_pipeline: pipeline,
            target_texture_handle: render_texture,
        }
    }

    pub fn reconfigure(&mut self, _config: &ConfigVariables) {}

    pub fn update(&mut self, _context: &mut Context, _delta_time_s: f32) {}

    pub fn draw(&mut self, context: &mut Context) {
        self.block.draw(context, self.geometry_pipeline);
    }

    pub fn get_target_texture(&self) -> TextureHandle {
        self.target_texture_handle
    }
}