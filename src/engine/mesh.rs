use std::collections::HashMap;

use cgmath::{Vector2, Vector3};

use crate::engine::datatypes::{ColoredVertex, Mesh, SimpleVertex, TexturedVertex};
use crate::renderer::context::Context;
use crate::renderer::types::VertexData;


#[repr(u32)]
pub enum PredefinedMesh {
    SimpleTriangle = 0,
    SimpleQuad = 1,
    _SimpleCube = 2,

    ColoredTriangle = 3,
    ColoredQuad = 4,
    _ColoredCube = 5,

    TexturedQuad = 6,
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
            let colored_vertex_buffer = context.create_static_vertex_buffer_sync(&colored_vertices);
            let simple_vertex_buffer = context.create_static_vertex_buffer_sync(&simple_vertices);
            let index_buffer = context.create_static_index_buffer_sync(&indices);

            let simple_mesh = Mesh {
                vertex_data: VertexData {
                    vertex_buffer: simple_vertex_buffer,
                    index_buffer,
                    index_count: indices.len() as u32
                }
            };
            let colored_mesh = Mesh {
                vertex_data: VertexData {
                    vertex_buffer: colored_vertex_buffer,
                    index_buffer,
                    index_count: indices.len() as u32
                }
            };
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
            let colored_vertex_buffer = context.create_static_vertex_buffer_sync(&colored_vertices);
            let simple_vertex_buffer = context.create_static_vertex_buffer_sync(&simple_vertices);
            let textured_vertex_buffer = context.create_static_vertex_buffer_sync(&textured_vertices);
            let index_buffer = context.create_static_index_buffer_sync(&indices);
            let simple_mesh = Mesh {
                vertex_data: VertexData {
                    vertex_buffer: simple_vertex_buffer,
                    index_buffer,
                    index_count: indices.len() as u32
                }
            };
            let colored_mesh = Mesh {
                vertex_data: VertexData {
                    vertex_buffer: colored_vertex_buffer,
                    index_buffer,
                    index_count: indices.len() as u32
                }
            };
            let textured_mesh = Mesh {
                vertex_data: VertexData {
                    vertex_buffer: textured_vertex_buffer,
                    index_buffer,
                    index_count: indices.len() as u32
                }
            };
            self.meshes
                .insert(PredefinedMesh::SimpleQuad as MeshHandle, simple_mesh);
            self.meshes
                .insert(PredefinedMesh::ColoredQuad as MeshHandle, colored_mesh);
            self.meshes
                .insert(PredefinedMesh::TexturedQuad as MeshHandle, textured_mesh);
        }
    }
}
