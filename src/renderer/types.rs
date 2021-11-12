use crate::renderer::buffer::BufferObjectHandle;
use crate::renderer::rawarray::RawArrayPtr;
use crate::renderer::types::VertexData::Buffered;
use ash::vk;
use ash::vk::{ImageView, PrimitiveTopology, Sampler};

//
// Render pass
//
pub const SWAPCHAIN_PASS: RenderPassHandle = 100_000;

pub type RenderPassHandle = u32;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PipelineHandle {
    pub(super) render_pass: RenderPassHandle,
    pipeline_index: u32,
}

impl PipelineHandle {
    pub fn new(render_pass: RenderPassHandle, pipeline_index: u32) -> Self {
        PipelineHandle {
            render_pass,
            pipeline_index,
        }
    }

    pub(super) fn index(&self) -> usize {
        self.pipeline_index as usize
    }
}

//
// Texture
//
pub type TextureHandle = usize;
pub type SamplerHandle = usize;

// Pipeline
pub type UniformHandle = usize;

pub trait VertexInputDescription {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

#[derive(Clone, Debug, Copy)]
pub enum VertexTopology {
    Triangle,
    TriangeStrip,
}

pub struct PipelineConfiguration {
    pub(super) vertex_shader_code: Vec<u8>,
    pub(super) fragment_shader_code: Vec<u8>,
    pub(super) push_constant_buffer_size: Option<usize>,
    pub(super) vertex_topology: VertexTopology,
    pub(super) vertex_uniform_cfg: Option<BufferObjectConfiguration>,
    pub(super) fragment_uniform_cfg: Option<BufferObjectConfiguration>,
    pub(super) storage_buffer_cfg: Option<BufferObjectConfiguration>,
    pub(super) texture_cfgs: Vec<TextureConfiguration>,
    pub(super) alpha_blending: bool,
}

impl PipelineConfiguration {
    pub fn builder() -> PipelineConfigurationBuilder {
        PipelineConfigurationBuilder {
            vertex_shader_code: Option::None,
            fragment_shader_code: Option::None,
            push_constant_buffer_size: Option::None,
            vertex_topology: Option::None,
            vertex_uniform_cfg: Option::None,
            fragment_uniform_cfg: Option::None,
            storage_buffer_cfg: Option::None,
            texture_cfgs: Vec::new(),
            alpha_blending: false,
        }
    }
}

pub struct PipelineConfigurationBuilder {
    vertex_shader_code: Option<Vec<u8>>,
    fragment_shader_code: Option<Vec<u8>>,
    push_constant_buffer_size: Option<usize>,
    vertex_topology: Option<VertexTopology>,
    vertex_uniform_cfg: Option<BufferObjectConfiguration>,
    fragment_uniform_cfg: Option<BufferObjectConfiguration>,
    storage_buffer_cfg: Option<BufferObjectConfiguration>,
    texture_cfgs: Vec<TextureConfiguration>,
    alpha_blending: bool,
}

impl PipelineConfigurationBuilder {
    pub fn with_fragment_shader(&mut self, code: Vec<u8>) -> &mut Self {
        self.fragment_shader_code = Some(code);

        self
    }

    pub fn with_vertex_shader(&mut self, code: Vec<u8>) -> &mut Self {
        self.vertex_shader_code = Some(code);

        self
    }

    pub fn with_push_constant<T>(&mut self) -> &mut Self {
        self.push_constant_buffer_size = Some(std::mem::size_of::<T>());

        self
    }

    pub fn with_vertex_topology(&mut self, vertex_topology: VertexTopology) -> &mut Self {
        self.vertex_topology = Some(vertex_topology);

        self
    }

    pub fn with_vertex_uniform(&mut self, binding: u8, buffer_object_handle: BufferObjectHandle) -> &mut Self {
        self.vertex_uniform_cfg = Some(BufferObjectConfiguration::new(binding, buffer_object_handle));

        self
    }

    pub fn with_fragment_uniform(&mut self, binding: u8, buffer_object_handle: BufferObjectHandle) -> &mut Self {
        self.fragment_uniform_cfg = Some(BufferObjectConfiguration::new(binding, buffer_object_handle));

        self
    }

    pub fn with_storage_buffer_object(&mut self, binding: u8, buffer_object_handle: BufferObjectHandle) -> &mut Self {
        self.storage_buffer_cfg = Some(BufferObjectConfiguration::new(binding, buffer_object_handle));

        self
    }

    pub fn with_alpha_blending(&mut self) -> &mut Self {
        self.alpha_blending = true;

        self
    }

    pub fn add_texture(&mut self, binding: u8, texture: TextureHandle, sampler: SamplerHandle) -> &mut Self {
        self.texture_cfgs
            .push(TextureConfiguration::new(binding, texture, sampler));

        self
    }

    pub fn build(&mut self) -> PipelineConfiguration {
        // TODO Load a default shader if not present
        let vertex_shader_code = self.vertex_shader_code.as_ref().expect("error").clone();
        let fragment_shader_code = self.fragment_shader_code.as_ref().expect("error").clone();

        let vertex_topology = self.vertex_topology.unwrap_or(VertexTopology::Triangle);

        PipelineConfiguration {
            vertex_shader_code,
            fragment_shader_code,
            push_constant_buffer_size: self.push_constant_buffer_size.take(),
            vertex_topology,
            vertex_uniform_cfg: self.vertex_uniform_cfg,
            fragment_uniform_cfg: self.fragment_uniform_cfg,
            storage_buffer_cfg: self.storage_buffer_cfg,
            texture_cfgs: self.texture_cfgs.clone(),
            alpha_blending: self.alpha_blending,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct BufferObjectConfiguration {
    pub(super) binding: u8,
    pub(super) buffer_object_handle: BufferObjectHandle,
}

impl BufferObjectConfiguration {
    pub fn new(binding: u8, buffer_object_handle: BufferObjectHandle) -> Self {
        BufferObjectConfiguration {
            binding,
            buffer_object_handle,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct TextureConfiguration {
    pub(super) binding: u8,
    pub(super) texture: TextureHandle,
    pub(super) sampler: SamplerHandle,
}

impl TextureConfiguration {
    pub fn new(binding: u8, texture: TextureHandle, sampler: SamplerHandle) -> Self {
        TextureConfiguration {
            binding,
            texture,
            sampler,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct SamplerBindingConfiguration {
    pub(super) binding: u8,
    pub(super) image: ImageView,
    pub(super) sampler: Sampler,
}

impl SamplerBindingConfiguration {
    pub fn new(binding: u8, image: ImageView, sampler: Sampler) -> Self {
        SamplerBindingConfiguration {
            binding,
            image,
            sampler,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub(super) struct BufferObjectBindingConfiguration {
    pub binding: u8,
    pub size: usize,
}

impl BufferObjectBindingConfiguration {
    pub(super) fn new(binding: u8, size: usize) -> Self {
        BufferObjectBindingConfiguration { binding, size }
    }
}

pub struct DrawCommand {
    pub pipeline: PipelineHandle,
    pub(super) push_constant_ptr: RawArrayPtr,
    pub(super) vertex_data: VertexData,
}

impl DrawCommand {
    pub fn new_buffered(
        pipeline: PipelineHandle,
        push_constant_ptr: RawArrayPtr,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        index_count: u32,
        instance_count: u32,
        instance_start: u32,
    ) -> DrawCommand {
        DrawCommand {
            pipeline,
            push_constant_ptr,
            vertex_data: Buffered(BufferData::new(
                vertex_buffer,
                index_buffer,
                index_count,
                instance_count,
                instance_start,
            )),
        }
    }

    pub fn triangle_count(&self, primitive_topology: PrimitiveTopology) -> u32 {
        match primitive_topology {
            PrimitiveTopology::TRIANGLE_LIST => match &self.vertex_data {
                Buffered(buffer_data) => buffer_data.index_count / 3 * buffer_data.instance_count,
            },
            PrimitiveTopology::TRIANGLE_STRIP => match &self.vertex_data {
                Buffered(buffer_data) => (buffer_data.index_count - 2) * buffer_data.instance_count,
            },
            _ => unreachable!(),
        }
    }
}

pub(super) struct BufferData {
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
    pub index_count: u32,
    pub instance_count: u32,
    pub instance_start: u32,
}

impl BufferData {
    pub(super) fn new(
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        index_count: u32,
        instance_count: u32,
        instance_start: u32,
    ) -> Self {
        BufferData {
            vertex_buffer,
            index_buffer,
            index_count,
            instance_count,
            instance_start,
        }
    }
}

pub(super) enum VertexData {
    Buffered(BufferData),
}

pub type Index = u32;

#[derive(Clone, Debug, Copy)]
pub enum UniformStage {
    Vertex,
    Fragment,
}
