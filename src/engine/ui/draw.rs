use crate::engine::datatypes::TexturedColoredVertex2D;

use crate::renderer::buffer::BufferObjectHandle;
use crate::renderer::context::Context;
use cgmath::{Vector2, Vector4, Zero};

pub fn draw_quad(
    context: &mut Context,
    handle: BufferObjectHandle,
    position: Vector2<u32>,
    extent: Vector2<u32>,
    color: Vector4<f32>,
) {
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            Vector2::new(position.x as f32 + 0f32, (position.y + extent.y) as f32),
            color,
            Vector2::zero(),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            Vector2::new((position.x + extent.x) as f32, (position.y + extent.y) as f32),
            color,
            Vector2::zero(),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            Vector2::new(position.x as f32, position.y as f32),
            color,
            Vector2::zero(),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            Vector2::new(position.x as f32, position.y as f32),
            color,
            Vector2::zero(),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            Vector2::new((position.x + extent.x) as f32, (position.y + extent.y) as f32),
            color,
            Vector2::zero(),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            Vector2::new((position.x + extent.x) as f32, position.y as f32),
            color,
            Vector2::zero(),
        ),
    );
}

pub fn draw_text(
    context: &mut Context,
    handle: BufferObjectHandle,
    text: &str,
    position: Vector2<u32>,
    char_size_px: u32,
    color: Vector4<f32>,
) {
    for (i, char) in text.chars().enumerate() {
        let char_position = Vector2::new((position.x + (i as u32 * char_size_px)) as f32, position.y as f32);
        draw_character(context, handle, char_position, color, char_size_px as f32, char);
    }
}

pub fn draw_text_shadowed(
    context: &mut Context,
    handle: BufferObjectHandle,
    text: &str,
    position: Vector2<u32>,
    text_size_px: u32,
    color: Vector4<f32>,
    shadow_color: Vector4<f32>,
) {
    draw_text(
        context,
        handle,
        text,
        Vector2::new(position.x + 2, position.y - 2),
        text_size_px,
        shadow_color,
    );
    draw_text(context, handle, text, position, text_size_px, color);
}

pub fn draw_character(
    context: &mut Context,
    handle: BufferObjectHandle,
    position: Vector2<f32>,
    color: Vector4<f32>,
    char_size: f32,
    char: char,
) {
    const WIDTH: u32 = 16;
    const TEXTURE_CHAR_WIDTH: f32 = 1.0 / 16.0;
    const TEXTURE_CHAR_HEIGHT: f32 = 1.0 / 6.0;

    // First ASCII character in the texture will be 32
    let char_u32 = char as u32 - 32;
    let offset_y = char_u32 / WIDTH;
    let offset_x = char_u32 % WIDTH;
    let offset = Vector2::new(
        offset_x as f32 * TEXTURE_CHAR_WIDTH,
        offset_y as f32 * TEXTURE_CHAR_HEIGHT,
    );

    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(position + Vector2::new(0f32, char_size), color, offset),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            position + Vector2::new(char_size, char_size),
            color,
            offset + Vector2::new(TEXTURE_CHAR_WIDTH, 0.0),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            position + Vector2::new(0f32, 0f32),
            color,
            offset + Vector2::new(0.0, TEXTURE_CHAR_HEIGHT),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            position + Vector2::new(0f32, 0f32),
            color,
            offset + Vector2::new(0.0, TEXTURE_CHAR_HEIGHT),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            position + Vector2::new(char_size, char_size),
            color,
            offset + Vector2::new(TEXTURE_CHAR_WIDTH, 0.0),
        ),
    );
    context.push_to_dynamic_buf(
        handle,
        TexturedColoredVertex2D::new(
            position + Vector2::new(char_size, 0f32),
            color,
            offset + Vector2::new(TEXTURE_CHAR_WIDTH, TEXTURE_CHAR_HEIGHT),
        ),
    );
}
