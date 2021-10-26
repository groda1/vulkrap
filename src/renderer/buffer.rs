use crate::renderer::context::PipelineHandle;
use crate::renderer::memory::MemoryManager;
use crate::renderer::rawarray::{PushError, RawArray, RawArrayPtr};
use crate::renderer::uniform::UniformStage;
use ash::vk;

pub type BufferObjectHandle = usize;

pub(super) struct BufferObjectManager {
    image_count: usize,
    dynamic_buffers: Vec<BufferObject>,
}

impl BufferObjectManager {
    pub fn new(image_count: usize) -> BufferObjectManager {
        BufferObjectManager {
            image_count,
            dynamic_buffers: Vec::new(),
        }
    }

    pub fn create_buffer<T>(
        &mut self,
        device: &ash::Device,
        memory_manager: &mut MemoryManager,
        capacity: usize,
        buffer_object_type: BufferObjectType,
        is_growable: bool,
    ) -> BufferObjectHandle {
        let handle = self.dynamic_buffers.len();

        let mut dynamic_buffer = BufferObject::new::<T>(capacity, self.image_count, buffer_object_type, is_growable);
        dynamic_buffer.build(device, memory_manager, self.image_count);

        self.dynamic_buffers.push(dynamic_buffer);

        handle
    }

    pub fn push_to_buf<T>(&mut self, handle: BufferObjectHandle, data: T) -> Result<RawArrayPtr, PushError> {
        debug_assert!(self.dynamic_buffers.len() > handle);
        self.dynamic_buffers[handle].raw_array.push(data)
    }

    pub fn borrow_buffer(&self, handle: BufferObjectHandle) -> &BufferObject {
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

    pub fn reset_buffers(&mut self) {
        for buffer in self.dynamic_buffers.iter_mut() {
            buffer.raw_array.reset();
        }
    }

    pub fn handle_buffer_overflow(
        &mut self,
        device: &ash::Device,
        memory_manager: &mut MemoryManager,
        handle: BufferObjectHandle,
        image_count: usize,
    ) {
        debug_assert!(self.dynamic_buffers.len() > handle);
        log_debug!("dynamicbuffer overflow handle={}:", handle);
        self.dynamic_buffers[handle].handle_buffer_overflow(device, memory_manager, image_count);
    }

    pub fn destroy(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager) {
        for buffer in self.dynamic_buffers.iter_mut() {
            unsafe {
                buffer.destroy(device, memory_manager);
            }
        }
    }
}

pub(super) enum BufferObjectType {
    Uniform(UniformStage),
    Storage,
    Vertex,
}

pub(super) struct BufferObject {
    buffer_object_type: BufferObjectType,
    capacity_bytes: usize,
    staging_buffer: Vec<vk::Buffer>,
    device_buffer: Vec<vk::Buffer>,
    raw_array: RawArray,

    assigned_pipelines: Vec<PipelineHandle>,
    is_growable: bool,
}

impl BufferObject {
    pub fn new<T>(
        capacity: usize,
        image_count: usize,
        buffer_object_type: BufferObjectType,
        is_growable: bool,
    ) -> Self {
        let staging_buffer = Vec::with_capacity(image_count);
        let device_buffer = Vec::with_capacity(image_count);

        let capacity_bytes = capacity * std::mem::size_of::<T>();

        BufferObject {
            buffer_object_type,
            capacity_bytes,
            staging_buffer,
            device_buffer,
            raw_array: RawArray::new::<T>(capacity).unwrap(),
            assigned_pipelines: Vec::new(),
            is_growable,
        }
    }

    pub fn build(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager, image_count: usize) {
        for _i in 0..image_count {
            let staging_buf = memory_manager.create_staging_buffer(device, self.capacity_bytes as vk::DeviceSize);
            let device_buf = memory_manager.create_device_buffer(
                device,
                self.capacity_bytes as vk::DeviceSize,
                vk::BufferUsageFlags::VERTEX_BUFFER,
            );

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

    pub fn device(&self, image_index: usize) -> vk::Buffer {
        debug_assert!(self.device_buffer.len() > image_index);

        self.device_buffer[image_index]
    }

    pub fn borrow_rawarray(&self) -> &RawArray {
        &self.raw_array
    }

    pub fn handle_buffer_overflow(
        &mut self,
        device: &ash::Device,
        memory_manager: &mut MemoryManager,
        image_count: usize,
    ) {
        let new_cap = self.raw_array.len() * 2;
        self.raw_array.resize(new_cap).expect("Failed to resize dynamic buffer");
        self.capacity_bytes = self.capacity_bytes * 2;

        log_debug!(" - resized device buffers to {}", self.capacity_bytes);
        log_debug!(" - {:?}", self.raw_array);

        unsafe {
            device.device_wait_idle().expect("Failed to wait device idle!");
            self.destroy(device, memory_manager);
            self.build(device, memory_manager, image_count);
        }
    }
}
