use crate::renderer::types::{SamplerHandle, TextureHandle};
use ash::vk;
use ash::vk::{DeviceMemory, Image, ImageView, Sampler};
use std::ptr;

struct Texture {
    image: Image,
    image_memory: DeviceMemory,
    image_view: ImageView,

    width: u32,
    height: u32,
    format: vk::Format,
}

pub struct TextureManager {
    textures: Vec<Texture>,
    samplers: Vec<Sampler>,
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager {
            textures: Vec::new(),
            samplers: Vec::new(),
        }
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        for texture in self.textures.iter() {
            unsafe {
                device.destroy_image_view(texture.image_view, None);
                device.destroy_image(texture.image, None);
                device.free_memory(texture.image_memory, None);
            }
        }

        for sampler in self.samplers.iter() {
            unsafe {
                device.destroy_sampler(*sampler, None);
            }
        }
    }

    pub fn add_texture(&mut self,
                       image: Image,
                       image_memory: DeviceMemory,
                       image_view: ImageView,
                       width: u32,
                       height: u32,
                       format: vk::Format) -> TextureHandle {
        let handle = self.textures.len();

        let texture = Texture {
            image,
            image_memory,
            image_view,
            width,
            height,
            format
        };
        self.textures.push(texture);

        handle
    }

    pub fn add_sampler(&mut self, device: &ash::Device) -> SamplerHandle {
        let handle = self.samplers.len();

        let sampler = _create_texture_sampler(device);
        self.samplers.push(sampler);

        handle
    }

    pub fn get_imageview(&self, texture: TextureHandle) -> ImageView {
        debug_assert!(self.textures.len() > texture);

        self.textures[texture].image_view
    }

    pub fn get_extent(&self, texture: TextureHandle) -> (u32, u32) {
        debug_assert!(self.textures.len() > texture);

        let texture = &self.textures[texture];

        (texture.width, texture.height)
    }

    pub fn get_format(&self, texture: TextureHandle) -> vk::Format {
        debug_assert!(self.textures.len() > texture);

        self.textures[texture].format
    }

    pub fn get_sampler(&self, sampler: SamplerHandle) -> Sampler {
        debug_assert!(self.samplers.len() > sampler);

        self.samplers[sampler]
    }
}

fn _create_texture_sampler(device: &ash::Device) -> Sampler {
    let sampler_create_info = vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        max_anisotropy: 16.0,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        min_lod: 0.0,
        max_lod: 1.0,
        mip_lod_bias: 0.0,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        anisotropy_enable: vk::TRUE,
        unnormalized_coordinates: vk::FALSE,
    };

    unsafe {
        device
            .create_sampler(&sampler_create_info, None)
            .expect("Failed to create Sampler!")
    }
}
