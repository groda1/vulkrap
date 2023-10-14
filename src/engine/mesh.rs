use std::collections::HashMap;
use std::path::Path;

use cgmath::{Vector2, Vector3};

use crate::engine::datatypes::{ColoredVertex, Mesh, SimpleVertex, TexturedVertex, NormalVertex};
use crate::engine::model::obj;
use crate::renderer::context::Context;

#[repr(u32)]
pub enum PredefinedMesh {
    SimpleTriangle = 0,
    SimpleQuad = 1,
    SimpleCube = 2,

    NormaledTriangle = 3,
    NormaledQuad = 4,
    NormaledCube = 5,

    ColoredTriangle = 6,
    ColoredQuad = 7,
    ColoredCube = 8,

    TexturedTriangle = 9,
    TexturedQuad = 10,
}

pub type MeshHandle = u32;

const _START_HANDLE: MeshHandle = 1000;

pub struct MeshManager {
    meshes: HashMap<MeshHandle, Mesh>,
    next_handle: MeshHandle
}

impl MeshManager {
    pub fn new(context: &mut Context) -> MeshManager {
        let mut mesh_manager = MeshManager {
            meshes: HashMap::new(),
            next_handle: 1000
        };

        mesh_manager.load_predefined_meshes(context);
        mesh_manager
    }


    pub fn get_mesh(&self, mesh_handle: MeshHandle) -> &Mesh {
        self.meshes
            .get(&mesh_handle)
            .expect("Failed to fetch mesh")
    }

    pub fn load_new_mesh(&mut self, context: &mut Context, path: &Path) -> Result<MeshHandle, &'static str> {
        let extension = path.extension();
        if extension.is_none() {
            return Err("Unknown file type");
        }

        let extension = extension.unwrap();

        if extension == "obj" {
            let mesh = obj::load_obj_mesh(context, path).unwrap();
            let handle = self.next_handle;
            self.meshes.insert(handle ,mesh);
            self.next_handle = self.next_handle + 1;
            return Ok(handle);

        } else {

        }

        Err("failed to load mesh")

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
            let colored_vertex_buffer = context.create_static_vertex_buffer_sync(&colored_vertices);
            let simple_vertex_buffer = context.create_static_vertex_buffer_sync(&simple_vertices);
            let index_buffer = context.create_static_index_buffer_sync(&indices);

            let simple_mesh = Mesh::new(simple_vertex_buffer, index_buffer, indices.len() as u32);
            let colored_mesh = Mesh::new(colored_vertex_buffer, index_buffer, indices.len() as u32);
            self.meshes
                .insert(PredefinedMesh::SimpleTriangle as MeshHandle, simple_mesh);
            self.meshes
                .insert(PredefinedMesh::ColoredTriangle as MeshHandle, colored_mesh);
        }

        {
            let simple_vertices = vec![
                SimpleVertex::new(Vector3::new(-0.5, 0.5, 0.0)),
                SimpleVertex::new(Vector3::new(0.5, 0.5, 0.0)),
                SimpleVertex::new(Vector3::new(-0.5, -0.5, 0.0)),
                SimpleVertex::new(Vector3::new(0.5, -0.5, 0.0)),
            ];
            let normaled_vertices = vec![
                NormalVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                NormalVertex::new(Vector3::new(0.5, 0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                NormalVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                NormalVertex::new(Vector3::new(0.5, -0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            ];
            let colored_vertices = vec![
                ColoredVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(1.0, 0.0, 0.0)),
                ColoredVertex::new(Vector3::new(0.5, 0.5, 0.0), Vector3::new(0.0, 1.0, 0.0)),
                ColoredVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector3::new(0.0, 0.0, 1.0)),
                ColoredVertex::new(Vector3::new(0.5, -0.5, 0.0), Vector3::new(1.0, 0.0, 1.0)),
            ];
            let textured_vertices = vec![
                TexturedVertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector2::new(0.0, 0.0)),
                TexturedVertex::new(Vector3::new(0.5, 0.5, 0.0), Vector2::new(1.0, 0.0)),
                TexturedVertex::new(Vector3::new(-0.5, -0.5, 0.0), Vector2::new(0.0, 1.0)),
                TexturedVertex::new(Vector3::new(0.5, -0.5, 0.0), Vector2::new(1.0, 1.0)),
            ];

            let indices = vec![0, 1, 2, 2, 1, 3];
            let index_count = indices.len() as u32;
            let simple_vertex_buffer = context.create_static_vertex_buffer_sync(&simple_vertices);
            let normaled_vertex_buffer = context.create_static_vertex_buffer_sync(&normaled_vertices);
            let colored_vertex_buffer = context.create_static_vertex_buffer_sync(&colored_vertices);
            let textured_vertex_buffer = context.create_static_vertex_buffer_sync(&textured_vertices);
            let index_buffer = context.create_static_index_buffer_sync(&indices);

            let simple_mesh = Mesh::new(simple_vertex_buffer, index_buffer, index_count);
            let normaled_mesh = Mesh::new(normaled_vertex_buffer, index_buffer, index_count);
            let colored_mesh = Mesh::new(colored_vertex_buffer, index_buffer, index_count);
            let textured_mesh = Mesh::new(textured_vertex_buffer, index_buffer, index_count);

            self.meshes
                .insert(PredefinedMesh::SimpleQuad as MeshHandle, simple_mesh);
            self.meshes
                .insert(PredefinedMesh::NormaledQuad as MeshHandle, normaled_mesh);
            self.meshes
                .insert(PredefinedMesh::ColoredQuad as MeshHandle, colored_mesh);
            self.meshes
                .insert(PredefinedMesh::TexturedQuad as MeshHandle, textured_mesh);
        }
    }
}
