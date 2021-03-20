use std::ptr;

use cgmath::{InnerSpace, Vector3};
use num::Integer;

use crate::engine::datatypes::VertexNormal;
use crate::engine::mesh::Mesh;
use crate::renderer::context::Context;
use crate::renderer::pipeline::PipelineDrawCommand;

pub struct Terrain {
    chunk: Mesh,
}

impl Terrain {
    pub fn new(context: &mut Context) -> Self {
        let raw_vertices = create_raw_vertices(64, 64, sin_terrain);
        //for (i, vertex) in raw_vertices.iter().enumerate() {
        //    println!("raw vertex: {} {:?}", i, vertex);
        //}

        let chunk_data = create_flat_normaled_chunk(64, 64, &raw_vertices);

        let vertex_buffer = context.allocate_vertex_buffer(&chunk_data.vertices);
        let index_buffer = context.allocate_index_buffer(&chunk_data.indices);

        Terrain {
            chunk: Mesh::new(vertex_buffer, index_buffer, chunk_data.indices.len() as u32),
        }
    }

    pub fn set_draw_commands(&self, draw_command_buffer: &mut Vec<PipelineDrawCommand>) {
        draw_command_buffer.clear();

        draw_command_buffer.push(PipelineDrawCommand::new(
            self.chunk.vertex_buffer,
            self.chunk.index_buffer,
            self.chunk.index_count,
            ptr::null(),
        ));
    }
}

fn sin_terrain(x: f32, y: f32) -> f32 {
    let x1 = (x * 1.5).sin() * 0.3;
    let y1 = (y * 0.4).cos() * 1.0;

    let xy = (y + x * 0.3).sin() * 0.3;

    x1 + y1 + xy
}

struct ChunkData {
    vertices: Vec<VertexNormal>,
    indices: Vec<u32>,
}

fn create_flat_normaled_chunk(
    quad_count_width: usize,
    quad_count_height: usize,
    raw_vertices: &[Vector3<f32>],
) -> ChunkData {
    let strip_count = quad_count_height as usize;
    let vertices_per_strip = ((quad_count_width + 1) * 2) as usize;

    let mut vertices = Vec::with_capacity(strip_count * vertices_per_strip);
    let mut indices = Vec::with_capacity(strip_count * vertices_per_strip + 1);
    let mut normals = Vec::with_capacity(vertices.len());
    for i in 0..quad_count_height as usize {
        for j in 0..(quad_count_width + 1) as usize {
            indices.push(vertices.len() as u32);
            vertices.push(raw_vertices[j + i * (quad_count_width + 1)]);
            indices.push(vertices.len() as u32);
            vertices.push(raw_vertices[j + (i + 1) * (quad_count_width + 1)]);
        }
        indices.push(0xffffffff);
    }

    for i in 0..vertices.len() {
        // Last two vertices of each strip
        if (i % vertices_per_strip) >= (vertices_per_strip - 2) {
            normals.push(Vector3::new(0.0, 0.0, 0.0));
        } else {
            let v1 = vertices[i + 1] - vertices[i];
            let v2 = vertices[i + 2] - vertices[i];
            // v1 = v1.normalize();
            // v2 = v2.normalize();

            if i.is_even() {
                normals.push(v2.cross(v1).normalize());
            } else {
                normals.push(v1.cross(v2).normalize());
            }
        }
    }

    assert_eq!(vertices.len(), normals.len());
    assert_eq!(indices.len(), strip_count * (vertices_per_strip + 1));

    let mut complete_vertices = Vec::with_capacity(vertices.len());
    for (i, vertex) in vertices.iter().enumerate() {
        complete_vertices.push(VertexNormal::new(*vertex, normals[i]));
        //println!("vertices: {} {:?} [{:?}]", i, vertex, normals[i]);
    }
    //println!("indices: {:?}", indices);

    ChunkData {
        vertices: complete_vertices,
        indices,
    }
}

fn create_raw_vertices(
    quad_count_width: usize,
    quad_count_height: usize,
    height_function: fn(f32, f32) -> f32,
) -> Vec<Vector3<f32>> {
    let width = quad_count_width;
    let height = quad_count_height;

    let vertex_count = ((width + 1) * (height + 1)) as usize;

    let mut vertices = Vec::with_capacity(vertex_count);

    // Vertices
    for i in 0..(height + 1) {
        for j in 0..(width + 1) {
            let x_offset = j as f32 * 1.0;
            let z_offset = i as f32 * -1.0;

            let y = height_function(x_offset, -z_offset);

            vertices.push(Vector3::new(x_offset, y, z_offset));
        }
    }

    vertices
}

#[cfg(test)]
mod tests {
    use crate::engine::terrain::create_flat_normaled_chunk;

    #[test]
    fn test() {
        create_flat_normaled_chunk(3, 2);
    }
}
