use ash::vk;

pub struct SynchronizationHandler {
    image_available_semaphores: Vec<vk::Semaphore>,
    transfer_finished_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,

    inflight_counter: usize,
}

// TODO this needs to fixed
const MAX_FRAMES_IN_FLIGHT: usize = 2;

impl SynchronizationHandler {
    pub fn new(logical_device: &ash::Device) -> SynchronizationHandler {
        // TODO dont use MAX_FRAMES_IN_FLIGHT.
        // TODO Use the ACTUAL! number of images in the swapchain, and fix recreate function so that it works if it changes.
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut transfer_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut inflight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = logical_device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let transfer_finished_semaphore = logical_device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = logical_device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = logical_device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                image_available_semaphores.push(image_available_semaphore);
                transfer_finished_semaphores.push(transfer_finished_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                inflight_fences.push(inflight_fence);
            }
        }

        SynchronizationHandler {
            image_available_semaphores,
            transfer_finished_semaphores,
            render_finished_semaphores,
            inflight_fences,

            inflight_counter: 0,
        }
    }

    pub unsafe fn destroy(&mut self, logical_device: &ash::Device) {
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            logical_device.destroy_semaphore(self.image_available_semaphores[i], None);
            logical_device.destroy_semaphore(self.transfer_finished_semaphores[i], None);
            logical_device.destroy_semaphore(self.render_finished_semaphores[i], None);
            logical_device.destroy_fence(self.inflight_fences[i], None);
        }
    }

    pub fn image_available_semaphore(&self) -> vk::Semaphore {
        self.image_available_semaphores[self.inflight_counter]
    }

    pub fn transfer_finished_semaphore(&self) -> vk::Semaphore {
        self.transfer_finished_semaphores[self.inflight_counter]
    }

    pub fn render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphores[self.inflight_counter]
    }

    pub fn inflight_fence(&self) -> vk::Fence {
        self.inflight_fences[self.inflight_counter]
    }

    pub fn step(&mut self) {
        self.inflight_counter = (self.inflight_counter + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}
