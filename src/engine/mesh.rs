use crate::engine::datatypes::ColoredVertex;
use crate::engine::mesh::PredefinedMesh::{QUAD, TRIANGLE};
use crate::renderer::context::Context;
use ash::vk;
use cgmath::Vector3;
use std::collections::HashMap;

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
    pub index_count: u32,
}

#[repr(u32)]
pub enum PredefinedMesh {
    TRIANGLE = 0,
    QUAD = 1,
    //CUBE = 2,
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
        let vertices = vec![
            ColoredVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector3::new(1.0, 0.0, 0.0)),
            ColoredVertex::new(Vector3::new(0.5, -0.5, 0.0), Vector3::new(0.0, 1.0, 0.0)),
            ColoredVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
        ];
        let indices = vec![0, 1, 2];
        let vertex_buffer = context.allocate_vertex_buffer(&vertices);
        let index_buffer = context.allocate_index_buffer(&indices);
        let mesh = Mesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        };

        self.meshes.insert(TRIANGLE as MeshHandle, mesh);

        let vertices = vec![
            ColoredVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector3::new(1.0, 0.0, 0.0)),
            ColoredVertex::new(Vector3::new(0.5, -0.5, 0.0), Vector3::new(0.0, 1.0, 0.0)),
            ColoredVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            ColoredVertex::new(Vector3::new(0.5, 0.5, 0.0), Vector3::new(1.0, 0.0, 1.0)),
        ];
        let indices = vec![0, 1, 2, 2, 1, 3];
        let vertex_buffer = context.allocate_vertex_buffer(&vertices);
        let index_buffer = context.allocate_index_buffer(&indices);
        let mesh = Mesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        };

        self.meshes.insert(QUAD as MeshHandle, mesh);
    }
}
