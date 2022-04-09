use std::ptr;

use cgmath::{InnerSpace, Vector3};
use num::Integer;

use crate::engine::datatypes::VertexNormal;
use crate::renderer::context::Context;
use crate::renderer::types::{DrawCommand, VertexData, PipelineHandle};

const QUAD_SIZE: f32 = 1.0;

pub struct _OctreeTerrainNode {
    size: f32, // quad_width_count * (adjusted for LOD QUAD_SIZE)
    center_point: Vector3<f32>,

    mesh: VertexData,

    // Order: 0 = SW, 1 = SE, 2 = NW, 3 = NE
    children: Option<Box<[_OctreeTerrainNode; 4]>>,
}

pub struct Terrain {
    pipeline: PipelineHandle,

    chunk: VertexData,
}

impl Terrain {
    pub fn new(context: &mut Context, pipeline: PipelineHandle) -> Self {
        let quad_width = 256;
        let quad_height = quad_width;

        debug_assert_eq!(quad_width % 64, 0);

        let raw_vertices = create_raw_vertices(quad_width, quad_height, sin_terrain);
        //for (i, vertex) in raw_vertices.iter().enumerate() {
        //    println!("raw vertex: {} {:?}", i, vertex);
        //}

        //
        let chunk_data = create_flat_normaled_chunk(quad_width, quad_height, &raw_vertices);

        let vertex_buffer = context.create_static_vertex_buffer_sync(&chunk_data.vertices);
        let index_buffer = context.create_static_index_buffer_sync(&chunk_data.indices);

        Terrain {
            pipeline,
            chunk: VertexData::new(vertex_buffer, index_buffer, chunk_data.indices.len() as u32),
        }
    }

    pub fn draw(&self, context: &mut Context) {
        context.add_draw_command(DrawCommand::new_buffered(
            self.pipeline,
            ptr::null(),
            self.chunk,
            1,
            0,
        ));
    }
}

fn sin_terrain(x: f32, y: f32, scale: u8) -> f32 {
    let x_scaled = x * 64.0;
    let y_scaled = y * 64.0;

    let mut x1 = (x_scaled * 1.5).sin() * 0.3;
    let mut y1 = (y_scaled * 0.4).cos() * 1.0;

    let mut xy = (y_scaled + x_scaled * 0.3).sin() * 0.3;

    let mut bonus = 0.0;
    if x > 0.45 && x < 0.55 && y > 0.45 && y < 0.55 {
        bonus += 2.5;
    }
    if x > 0.48 && x < 0.52 && y > 0.48 && y < 0.52 {
        bonus += 2.5;
        x1 = 0.0;
        y1 = 0.0;
        xy = 0.0;
    }

    (x1 + y1 + xy + bonus) * scale as f32
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
    height_function: fn(f32, f32, u8) -> f32,
) -> Vec<Vector3<f32>> {
    let width = quad_count_width;
    let height = quad_count_height;

    let vertex_count = ((width + 1) * (height + 1)) as usize;

    let mut vertices = Vec::with_capacity(vertex_count);

    // Vertices
    for i in 0..(height + 1) {
        for j in 0..(width + 1) {
            let x_offset = j as f32 * QUAD_SIZE;
            let z_offset = i as f32 * -QUAD_SIZE;

            let normalized_x_offset = j as f32 / quad_count_width as f32;
            let normalized_y_offset = i as f32 / quad_count_height as f32;

            let y = height_function(normalized_x_offset, normalized_y_offset, (quad_count_width / 64) as u8);

            vertices.push(Vector3::new(x_offset, y, z_offset));
        }
    }

    vertices
}
