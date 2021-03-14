use std::collections::HashMap;

use ash::vk;
use cgmath::Vector3;

use crate::engine::datatypes::{ColoredVertex, SimpleVertex};
use crate::renderer::context::Context;

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
    pub index_count: u32,
}

#[repr(u32)]
pub enum PredefinedMesh {
    SimpleTriangle = 0,
    SimpleQuad = 1,
    _SimpleCube = 2,

    ColoredTriangle = 3,
    ColoredQuad = 4,
    _ColoredCube = 5,
}

type MeshHandle = u32;

const _START_HANDLE: MeshHandle = 1000;

pub struct MeshManager {
    meshes: HashMap<MeshHandle, Mesh>,
}

impl MeshManager {
    pub fn new(context: &mut Context) -> MeshManager {
        let mut mesh_manager = MeshManager { meshes: HashMap::new() };
        mesh_manager.load_predefined_meshes(context);

        mesh_manager
    }

    pub fn _get_mesh(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(&handle)
    }

    pub fn get_predefined_mesh(&self, predefined: PredefinedMesh) -> &Mesh {
        self.meshes
            .get(&(predefined as MeshHandle))
            .expect("Failed to fetch predefined mesh")
    }

    fn load_predefined_meshes(&mut self, context: &mut Context) {
        {
            let colored_vertices = vec![
                ColoredVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(1.0, 0.0, 0.0)),
                ColoredVertex::new(Vector3::new(0.5, 0.5, 0.0), Vector3::new(0.0, 1.0, 0.0)),
                ColoredVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            ];
            let simple_vertices = vec![
                SimpleVertex::new(Vector3::new(-0.5, 0.5, 0.0)),
                SimpleVertex::new(Vector3::new(0.5, 0.5, 0.0)),
                SimpleVertex::new(Vector3::new(-0.5, -0.5, 0.0)),
            ];
            let indices = vec![0, 1, 2];
            let colored_vertex_buffer = context.allocate_vertex_buffer(&colored_vertices);
            let simple_vertex_buffer = context.allocate_vertex_buffer(&simple_vertices);
            let index_buffer = context.allocate_index_buffer(&indices);
            let simple_mesh = Mesh {
                vertex_buffer: simple_vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            };
            let colored_mesh = Mesh {
                vertex_buffer: colored_vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            };
            self.meshes
                .insert(PredefinedMesh::SimpleTriangle as MeshHandle, simple_mesh);
            self.meshes
                .insert(PredefinedMesh::ColoredTriangle as MeshHandle, colored_mesh);
        }

        {
            let colored_vertices = vec![
                ColoredVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(1.0, 0.0, 0.0)),
                ColoredVertex::new(Vector3::new(0.5, 0.5, 0.0), Vector3::new(0.0, 1.0, 0.0)),
                ColoredVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                ColoredVertex::new(Vector3::new(0.5, -0.5, 0.0), Vector3::new(1.0, 0.0, 1.0)),
            ];
            let simple_vertices = vec![
                SimpleVertex::new(Vector3::new(-0.5, 0.5, 0.0)),
                SimpleVertex::new(Vector3::new(0.5, 0.5, 0.0)),
                SimpleVertex::new(Vector3::new(-0.5, -0.5, 0.0)),
                SimpleVertex::new(Vector3::new(0.5, -0.5, 0.0)),
            ];
            let indices = vec![0, 1, 2, 2, 1, 3];
            let colored_vertex_buffer = context.allocate_vertex_buffer(&colored_vertices);
            let simple_vertex_buffer = context.allocate_vertex_buffer(&simple_vertices);
            let index_buffer = context.allocate_index_buffer(&indices);
            let simple_mesh = Mesh {
                vertex_buffer: simple_vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            };
            let colored_mesh = Mesh {
                vertex_buffer: colored_vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            };
            self.meshes
                .insert(PredefinedMesh::SimpleQuad as MeshHandle, simple_mesh);
            self.meshes
                .insert(PredefinedMesh::ColoredQuad as MeshHandle, colored_mesh);
        }
    }
}
