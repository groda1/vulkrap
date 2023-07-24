use ash::vk;
use ash::vk::SwapchainKHR;

pub enum RenderTarget {
    ImageTarget(ImageTarget),
    SwapchainTarget(SwapchainTarget),
}

impl RenderTarget {
    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        match self {
            RenderTarget::ImageTarget(image) => { image.destroy(device); }
            RenderTarget::SwapchainTarget(swapchain) => { swapchain.destroy(device); }
        }
    }

    pub fn swapchain_target(&self) -> Option<&SwapchainTarget> {
        match self {
            RenderTarget::SwapchainTarget(swapchain) => Some(swapchain),
            _ => None
        }
    }

    pub fn image_count(&self) -> usize {
        match self {
            RenderTarget::ImageTarget(_) => { 1 }
            RenderTarget::SwapchainTarget(swapchain) => { swapchain.image_count() }
        }
    }

    pub fn framebuffer(&self, image_index: usize) -> vk::Framebuffer {
        match self {
            RenderTarget::ImageTarget(image) => { image.framebuffer }
            RenderTarget::SwapchainTarget(swapchain) => { swapchain.framebuffers[image_index] }
        }
    }
}

pub struct ImageTarget {
    /* TODO: I dont think we need to keep any of this as all this will is owned by the texture manager */
    color_image: vk::Image,
    color_image_view: vk::ImageView,
    color_image_memory: vk::DeviceMemory,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    framebuffer: vk::Framebuffer,
}

impl ImageTarget {
    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        unimplemented!()
    }
}

pub struct SwapchainTarget {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: SwapchainKHR,

    color_imageviews: Vec<vk::ImageView>,

    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,

    framebuffers: Vec<vk::Framebuffer>,
}

impl SwapchainTarget {
    pub fn new(
        swapchain_loader: ash::extensions::khr::Swapchain,
        swapchain: SwapchainKHR,
        color_imageviews: Vec<vk::ImageView>,
        depth_image: vk::Image,
        depth_image_view: vk::ImageView,
        depth_image_memory: vk::DeviceMemory,
        framebuffers: Vec<vk::Framebuffer>,
    ) -> Self {
        SwapchainTarget {
            swapchain_loader,
            swapchain,
            color_imageviews,
            depth_image,
            depth_image_view,
            depth_image_memory,
            framebuffers,
        }
    }

    unsafe fn destroy(&mut self, device: &ash::Device) {
        // Depth buffer
        device.destroy_image_view(self.depth_image_view, None);
        device.destroy_image(self.depth_image, None);
        device.free_memory(self.depth_image_memory, None);

        // Color buffers
        for color_imageview in self.color_imageviews.iter() {
            device.destroy_image_view(*color_imageview, None);
        }

        // Framebuffers
        for framebuffer in self.framebuffers.iter() {
            device.destroy_framebuffer(*framebuffer, None);
        }
        self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }

    pub fn loader(&self) -> &ash::extensions::khr::Swapchain {
        &self.swapchain_loader
    }

    pub fn swapchain(&self) -> SwapchainKHR {
        self.swapchain
    }

    pub fn image_count(&self) -> usize {
        debug_assert!(self.color_imageviews.len() == self.framebuffers.len());
        self.color_imageviews.len()
    }
}
