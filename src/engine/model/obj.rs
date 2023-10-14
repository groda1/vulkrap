use std::collections::HashMap;
use std::path::Path;
use std::str::{FromStr, SplitAsciiWhitespace};
use cgmath::Vector3;
use regex::Regex;
use stopwatch::Stopwatch;

use crate::engine::datatypes::{Mesh, NormalVertex};
use crate::renderer::context::Context;
use crate::util::file::read_lines;

#[derive(Debug)]
struct Face {
    vertices: Vector3<u32>,
    normals: Option<Vector3<u32>>,
    texture: Option<Vector3<u32>>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
struct VertexKey {
    vertex_index: u32,
    normal_index: Option<u32>,
    texture_index: Option<u32>,
}

pub fn load_obj_mesh(context: &mut Context, path: &Path) -> Result<Mesh, &'static str> {
    let sw = Stopwatch::start_new();
    log_debug!("loading obj_mesh: {:?}", path);
    let mut found_object = false;

    let mut raw_vertices = Vec::new();
    let mut raw_normals = Vec::new();
    let mut faces = Vec::new();

    let face_pattern = Regex::new(r"(?m)^f (?P<v1>\d*)(/(?P<t1>\d*)(/(?P<n1>\d*))?)? (?P<v2>\d*)(/(?P<t2>\d*)(/(?P<n2>\d*))?)? (?P<v3>\d*)(/(?P<t3>\d*)(/(?P<n3>\d*))?)?$").unwrap();

    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(line_str) = line {
                if line_str.starts_with("o") {
                    if found_object {
                        return Err("Multiple object not supported");
                    } else {
                        found_object = true;
                        let mut split = line_str.split_ascii_whitespace();
                        split.next();
                        log_debug!("load_obj_mesh: found o = {}",split.next().unwrap());
                    }
                } else if line_str.starts_with("v ") {
                    let mut split = line_str.split_ascii_whitespace();
                    split.next();
                    let vertex = _parse_vec3(&mut split);
                    raw_vertices.push(vertex);
                } else if line_str.starts_with("vn ") {
                    let mut split = line_str.split_ascii_whitespace();
                    split.next();
                    let normal = _parse_vec3(&mut split);
                    raw_normals.push(normal);
                } else if line_str.starts_with("f ") {
                    let face = _parse_face_line(&face_pattern, &line_str);
                    faces.push(face.unwrap());
                }
            }
        }
    }

    log_debug!("load_obj_mesh: obj vertex count: {}", raw_vertices.len());
    log_debug!("load_obj_mesh: obj normal count: {}", raw_normals.len());
    log_debug!("load_obj_mesh: obj face count {}", faces.len());

    let mut normal_vertices = Vec::new();
    let mut normal_vertex_to_index = HashMap::new();
    let mut indices = Vec::new();

    for face in faces.iter() {
        for i in 0..3 {

            let v = face.vertices[i] - 1;
            let n = if face.normals.is_some() {
                Some(face.normals.unwrap()[i] - 1)
            } else {
                None
            };

            let vertex_key = VertexKey {
                vertex_index: v,
                normal_index: n,
                texture_index: None,
            };

            let existing_vertex = normal_vertex_to_index.get(&vertex_key);

            if let Some(index) = existing_vertex {
                indices.push(*index);
            } else {
                let index = normal_vertices.len() as u32;
                let vertex = *raw_vertices.get(vertex_key.vertex_index as usize).unwrap();
                let normal = *raw_normals.get(vertex_key.normal_index.unwrap() as usize).unwrap();
                let normal_vertex = NormalVertex::new(vertex, normal);
                normal_vertices.push(normal_vertex);
                normal_vertex_to_index.insert(vertex_key, index);
                indices.push(index)
            }
        }
    }

    log_debug!("load_obj_mesh: buf vertex count: {}", normal_vertices.len());
    log_debug!("load_obj_mesh: buf index count: {}", indices.len());

    let vertex_buffer = context.create_static_vertex_buffer_sync(&normal_vertices);
    let index_buffer = context.create_static_index_buffer_sync(&indices);

    log_info!("loaded model in {} ms", sw.elapsed_ms());

    Ok(Mesh::new(vertex_buffer, index_buffer, indices.len() as u32))
}

fn _parse_vec3(split: &mut SplitAsciiWhitespace) -> Vector3<f32> {
    let x = split.next().unwrap();
    let y = split.next().unwrap();
    let z = split.next().unwrap();

    Vector3::new(
        f32::from_str(x).unwrap(),
        f32::from_str(y).unwrap(),
        f32::from_str(z).unwrap())
}

fn _parse_face_line(pattern: &Regex, str: &String) -> Option<Face> {

    if let Some(face) = pattern.captures(str.as_str()) {
        let v1 = u32::from_str(face.name("v1").unwrap().as_str()).unwrap();
        let v2 = u32::from_str(face.name("v2").unwrap().as_str()).unwrap();
        let v3 = u32::from_str(face.name("v3").unwrap().as_str()).unwrap();
        let vertices = Vector3::from((v3, v2, v1));

        let normals = if face.name("n1").is_some() {
            let n1 = u32::from_str(face.name("n1").unwrap().as_str()).unwrap();
            let n2 = u32::from_str(face.name("n2").unwrap().as_str()).unwrap();
            let n3 = u32::from_str(face.name("n3").unwrap().as_str()).unwrap();

            Some(Vector3::from((n3,n2,n1)))
        } else {
            None
        };
        let face = Face {
            vertices,
            normals,
            texture: None,
        };

        return Some(face);
    }
    None
}