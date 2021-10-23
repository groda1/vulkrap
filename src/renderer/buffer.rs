use crate::renderer::memory::MemoryManager;
use ash::vk;

pub type DynamicBufferHandle = usize;

pub struct DynamicBufferManager {
    image_count: usize,
    dynamic_buffers: Vec<DynamicBuffer>,
}

impl DynamicBufferManager {
    pub fn new(image_count: usize) -> DynamicBufferManager {
        DynamicBufferManager {
            image_count,
            dynamic_buffers: Vec::new(),
        }
    }

    pub fn create_dynamic_buffer(
        &mut self,
        device: &ash::Device,
        memory_manager: &mut MemoryManager,
        capacity: usize,
    ) -> DynamicBufferHandle {
        let handle = self.dynamic_buffers.len();

        let mut dynamic_buffer = DynamicBuffer::new(capacity, self.image_count);
        dynamic_buffer.build(device, memory_manager, self.image_count);

        self.dynamic_buffers.push(dynamic_buffer);

        handle
    }

    pub fn get(&self, handle: DynamicBufferHandle) -> &DynamicBuffer {
        debug_assert!(self.dynamic_buffers.len() > handle);

        &self.dynamic_buffers[handle]
    }

    pub fn rebuild(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager, image_count: usize) {
        self.image_count = image_count;

        for buffer in self.dynamic_buffers.iter_mut() {
            unsafe {
                buffer.destroy(device, memory_manager);
                buffer.build(device, memory_manager, image_count);
            }
        }
    }

    pub fn destroy(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager) {
        for buffer in self.dynamic_buffers.iter_mut() {
            unsafe {
                buffer.destroy(device, memory_manager);
            }
        }
        self.dynamic_buffers.clear();
    }
}

pub struct DynamicBuffer {
    capacity: usize,
    staging_buffer: Vec<vk::Buffer>,
    device_buffer: Vec<vk::Buffer>,
}

impl DynamicBuffer {
    pub fn new(capacity: usize, image_count: usize) -> Self {
        let staging_buffer = Vec::with_capacity(image_count);
        let device_buffer = Vec::with_capacity(image_count);

        DynamicBuffer {
            capacity,
            staging_buffer,
            device_buffer,
        }
    }

    pub fn build(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager, image_count: usize) {
        for i in 0..image_count {
            let staging_buf = memory_manager.create_staging_buffer(device, self.capacity as vk::DeviceSize);
            let device_buf = memory_manager.create_device_buffer(device, self.capacity as vk::DeviceSize);

            self.staging_buffer.push(staging_buf);
            self.device_buffer.push(device_buf);
        }
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager) {
        for buf in self.staging_buffer.iter() {
            memory_manager.destroy_buffer(device, *buf);
        }
        self.staging_buffer.clear();

        for buf in self.device_buffer.iter() {
            memory_manager.destroy_buffer(device, *buf);
        }
        self.device_buffer.clear();
    }

    pub fn staging(&self, image_index: usize) -> vk::Buffer {
        debug_assert!(self.staging_buffer.len() > image_index);

        self.staging_buffer[image_index]
    }
}
