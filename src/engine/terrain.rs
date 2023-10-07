use cgmath::{InnerSpace, Vector3};
use noise::Add;
use noise::Constant;
use noise::Multiply;
use num::Integer;
use noise::NoiseFn;
use noise::Perlin;
use noise::ScalePoint;

use crate::engine::datatypes::VertexNormal;
use crate::renderer::context::Context;
use crate::renderer::types::{DrawCommand, VertexData, PipelineHandle};

const QUAD_SIZE: f32 = 1.0;

#[allow(dead_code)]
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

        let seed = 1337;

        let scale = ScalePoint::new(Perlin::new(seed))
            .set_x_scale(4.0)
            .set_y_scale(4.0);
        let pass_1 = Multiply::new(scale, Constant::new(40.0));

        let scale = ScalePoint::new(Perlin::new(5432))
        .set_x_scale(8.0)
        .set_y_scale(8.0);
        let pass_2 = Multiply::new(scale, Constant::new(20.0));

        let scale = ScalePoint::new(Perlin::new(123145))
        .set_x_scale(16.0)
        .set_y_scale(16.0);
        let pass_3 = Multiply::new(scale, Constant::new(10.0));


        let result = Add::new(pass_1, Add::new(pass_2, pass_3));


        let quad_width = 256;
        let quad_height = 256;

        debug_assert_eq!(quad_width % 64, 0);

        let raw_vertices = create_raw_vertices(quad_width, quad_height, &result);
        let chunk_data = create_flat_normaled_chunk(quad_width, quad_height, &raw_vertices);

        let vertex_buffer = context.create_static_vertex_buffer_sync(&chunk_data.vertices);
        let index_buffer = context.create_static_index_buffer_sync(&chunk_data.indices);

        Terrain {
            pipeline,
            chunk: VertexData::new(vertex_buffer, index_buffer, chunk_data.indices.len() as u32),
        }
    }

    pub fn draw(&self, context: &mut Context) {
        context.add_draw_command(DrawCommand::new_buffered_nopush(
            self.pipeline,
            self.chunk,
        ));
    }
}

/*
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
*/


struct ChunkData {
    vertices: Vec<VertexNormal>,
    indices: Vec<u32>,
}

fn create_flat_normaled_chunk(
    quad_count_width: usize,
    quad_count_height: usize,
    raw_vertices: &[Vector3<f32>],
) -> ChunkData {
    let strip_count = quad_count_height;
    let vertices_per_strip = (quad_count_width + 1) * 2;

    let mut vertices = Vec::with_capacity(strip_count * vertices_per_strip);
    let mut indices = Vec::with_capacity(strip_count * vertices_per_strip + 1);
    let mut normals = Vec::with_capacity(vertices.len());
    for i in 0..quad_count_height {
        for j in 0..(quad_count_width + 1) {
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
    }

    ChunkData {
        vertices: complete_vertices,
        indices,
    }
}

fn create_raw_vertices<T: NoiseFn<f64, 2>>(
    quad_count_width: usize,
    quad_count_height: usize,
    noise_fn: &T,
) -> Vec<Vector3<f32>> {
    let width = quad_count_width;
    let height = quad_count_height;

    let vertex_count = (width + 1) * (height + 1);

    let mut vertices = Vec::with_capacity(vertex_count);

    // Vertices
    for i in 0..(height + 1) {
        for j in 0..(width + 1) {
            let x_offset = j as f64 * QUAD_SIZE as f64;
            let z_offset = i as f64 * -QUAD_SIZE as f64;

            let x = (j as f64 / width as f64) - 0.5;
            let z = (i as f64 / height as f64) - 0.5;
            let y = noise_fn.get([x, z]);

            vertices.push(Vector3::new(x_offset as f32, y as f32, z_offset as f32));
        }
    }

    vertices
}
