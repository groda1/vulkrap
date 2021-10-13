use crate::engine::datatypes::{ModelColorPushConstant, TextPushConstant};
use crate::engine::mesh::Mesh;
use crate::renderer::context::{Context, PipelineHandle};
use crate::renderer::pipeline::PipelineDrawCommand;
use cgmath::{Matrix4, Vector2, Vector3, Vector4};

pub fn draw_quad(
    context: &mut Context,
    target_buf: &mut Vec<PipelineDrawCommand>,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    position: Vector2<u32>,
    extent: Vector2<u32>,
    color: Vector4<f32>,
) {
    let transform = Matrix4::from_translation(Vector3::new(
        (position.x + (extent.x / 2)) as f32,
        (position.y + (extent.y / 2)) as f32,
        0.0,
    )) * Matrix4::from_nonuniform_scale(extent.x as f32, extent.y as f32, 1.0);
    let push_constant_ptr = context.add_push_constant(pipeline, ModelColorPushConstant::new(transform, color));

    let draw_command = PipelineDrawCommand::new(
        pipeline,
        mesh.vertex_buffer,
        mesh.index_buffer,
        mesh.index_count,
        push_constant_ptr,
    );
    target_buf.push(draw_command);
}

pub fn draw_text(
    context: &mut Context,
    target_buf: &mut Vec<PipelineDrawCommand>,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    text: &str,
    position: Vector2<u32>,
    text_size_px: u32,
    color: Vector3<f32>,
) {
    let scale = Matrix4::from_scale(text_size_px as f32);

    for (i, char) in text.chars().enumerate() {
        let transform = Matrix4::from_translation(Vector3::new(
            (position.x + (i as u32 * text_size_px) + (text_size_px / 2)) as f32,
            (position.y + (text_size_px / 2)) as f32,
            0.0,
        )) * scale;
        target_buf.push(draw_character(context, pipeline, mesh, transform, color, char));
    }
}

pub fn draw_text_shadowed(
    context: &mut Context,
    target_buf: &mut Vec<PipelineDrawCommand>,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    text: &str,
    position: Vector2<u32>,
    text_size_px: u32,
    color: Vector3<f32>,
    shadow_color: Vector3<f32>,
) {
    draw_text(
        context,
        target_buf,
        pipeline,
        mesh,
        text,
        Vector2::new(position.x + 2, position.y - 2),
        text_size_px,
        shadow_color,
    );
    draw_text(context, target_buf, pipeline, mesh, text, position, text_size_px, color);
}

pub fn _draw_text_random_color(
    context: &mut Context,
    target_buf: &mut Vec<PipelineDrawCommand>,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    text: &str,
    position: Vector2<u32>,
    text_size_px: u32,
) {
    let scale = Matrix4::from_scale(text_size_px as f32);

    for (i, char) in text.chars().enumerate() {
        let transform = Matrix4::from_translation(Vector3::new(
            position.x as f32 + (i as u32 * text_size_px) as f32,
            position.y as f32,
            0.0,
        )) * scale;
        target_buf.push(draw_character(
            context,
            pipeline,
            mesh,
            transform,
            _random_color(),
            char,
        ));
    }
}

pub fn draw_character(
    context: &mut Context,
    pipeline: PipelineHandle,
    mesh: &Mesh,
    model_transform: Matrix4<f32>,
    color: Vector3<f32>,
    char: char,
) -> PipelineDrawCommand {
    let push_constant_ptr = context.add_push_constant(pipeline, TextPushConstant::new(model_transform, color, char));

    PipelineDrawCommand::new(
        pipeline,
        mesh.vertex_buffer,
        mesh.index_buffer,
        mesh.index_count,
        push_constant_ptr,
    )
}

fn _random_color() -> Vector3<f32> {
    Vector3::new(rand::random(), rand::random(), rand::random())
}
