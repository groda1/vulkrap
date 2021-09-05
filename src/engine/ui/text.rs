use crate::engine::datatypes::TextPushConstant;
use crate::engine::mesh::Mesh;
use crate::renderer::context::PipelineHandle;
use crate::renderer::pipeline::PipelineDrawCommand;
use cgmath::{Matrix4, Vector2, Vector3};

pub fn draw_text(
    target_buf: &mut Vec<PipelineDrawCommand>,
    push_constant_buf: &mut Vec<TextPushConstant>,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    text: &str,
    position: Vector2<u32>,
    text_size_px: u32,
    color : Vector3<f32>,
) {
    let scale = Matrix4::from_scale(text_size_px as f32);

    for (i, char) in text.chars().enumerate() {
        let transform = Matrix4::from_translation(Vector3::new(position.x as f32 + (i as u32 * text_size_px) as f32, position.y as f32, 0.0)) * scale;
        target_buf.push(draw_character(push_constant_buf, pipeline, mesh, transform, color, char));
    }
}

pub fn draw_character(
    push_constant_buf: &mut Vec<TextPushConstant>,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    model_transform: Matrix4<f32>,
    color : Vector3<f32>,
    char: char,
) -> PipelineDrawCommand {
    push_constant_buf.push(TextPushConstant::new(
        model_transform,
        color,
        char,
    ));

    PipelineDrawCommand::new(
        pipeline,
        mesh.vertex_buffer,
        mesh.index_buffer,
        mesh.index_count,
        &push_constant_buf[push_constant_buf.len() - 1] as *const TextPushConstant as *const u8,
    )
}
