use crate::engine::datatypes::{InstancedCharacter, InstancedQuad};

use crate::renderer::context::Context;
use crate::renderer::types::BufferObjectHandle;
use cgmath::{Vector2, Vector4};

pub fn draw_quad(
    context: &mut Context,
    handle: BufferObjectHandle,
    position: Vector2<u32>,
    extent: Vector2<u32>,
    color: Vector4<f32>,
) -> u32 {
    let quad = InstancedQuad::new(
        Vector2::new(
            (position.x + (extent.x / 2)) as f32,
            (position.y + (extent.y / 2)) as f32,
        ),
        Vector2::new(extent.x as f32, extent.y as f32),
        color,
    );
    context.push_to_buffer_object(handle, quad);

    1
}

pub fn draw_text(
    context: &mut Context,
    handle: BufferObjectHandle,
    text: &str,
    position: Vector2<u32>,
    char_size_px: u32,
    color: Vector4<f32>,
) -> u32 {
    for (i, char) in text.chars().enumerate() {
        let char_position = Vector2::new(
            (position.x + (char_size_px / 2) + (i as u32 * char_size_px)) as f32,
            (position.y + (char_size_px / 2)) as f32,
        );
        context.push_to_buffer_object(
            handle,
            InstancedCharacter::new(char_position, color, char as u32, char_size_px as f32),
        );
    }

    text.len() as u32
}

pub fn draw_text_shadowed(
    context: &mut Context,
    handle: BufferObjectHandle,
    text: &str,
    position: Vector2<u32>,
    char_size_px: u32,
    color: Vector4<f32>,
    shadow_color: Vector4<f32>,
) -> u32 {
    let mut instance_count = 0;
    instance_count += draw_text(
        context,
        handle,
        text,
        Vector2::new(position.x + 2, position.y - 2),
        char_size_px,
        shadow_color,
    );
    instance_count += draw_text(context, handle, text, position, char_size_px, color);

    instance_count
}
