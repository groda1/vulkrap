use std::slice::from_raw_parts;
use std::time::Instant;

use ash::vk;

use crate::renderer::context::PipelineHandle;
use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::{PipelineContainer, UniformStage};
use crate::renderer::rawarray::{PushError, RawArray, RawArrayPtr};
use crate::renderer::stats::RenderStats;

pub type BufferObjectHandle = usize;

pub(super) struct BufferObjectManager {
    image_count: usize,
    buffer_objects: Vec<BufferObject>,
}

impl BufferObjectManager {
    pub fn new(image_count: usize) -> BufferObjectManager {
        BufferObjectManager {
            image_count,
            buffer_objects: Vec::new(),
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
        let handle = self.buffer_objects.len();

        let mut dynamic_buffer = BufferObject::new::<T>(capacity, self.image_count, buffer_object_type, is_growable);
        dynamic_buffer.build(device, memory_manager, self.image_count);

        self.buffer_objects.push(dynamic_buffer);

        handle
    }

    pub fn push_to_buf<T>(&mut self, handle: BufferObjectHandle, data: T) -> Result<RawArrayPtr, PushError> {
        debug_assert!(self.buffer_objects.len() > handle);
        self.buffer_objects[handle].push(data)
    }

    pub fn borrow_buffer(&self, handle: BufferObjectHandle) -> &BufferObject {
        debug_assert!(self.buffer_objects.len() > handle);
        &self.buffer_objects[handle]
    }

    pub fn reset_buffer(&mut self, handle: BufferObjectHandle) {
        debug_assert!(self.buffer_objects.len() > handle);
        self.buffer_objects[handle].reset();
    }

    pub fn rebuild(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager, image_count: usize) {
        self.image_count = image_count;

        for buffer in self.buffer_objects.iter_mut() {
            unsafe {
                buffer.destroy(device, memory_manager);
                buffer.build(device, memory_manager, image_count);
            }
        }
    }

    pub fn reassign_pipeline_buffers(&self, pipelines: &mut Vec<PipelineContainer>) {
        for buffer_object in self.buffer_objects.iter() {
            if let BufferObjectType::Uniform(stage) = buffer_object.buffer_object_type {
                for pipeline in buffer_object.assigned_pipelines.iter() {
                    pipelines[*pipeline].set_uniform_buffers(stage, buffer_object.devices());
                }
            } else if let BufferObjectType::Storage = buffer_object.buffer_object_type {
                unimplemented!()
            }
        }
    }

    pub fn assign_pipeline(&mut self, bo_handle: BufferObjectHandle, pipeline_handle: PipelineHandle) {
        debug_assert!(self.buffer_objects.len() > bo_handle);

        self.buffer_objects[bo_handle].assign_pipeline(pipeline_handle);
    }

    pub fn bake_command_buffer(
        &mut self,
        logical_device: &ash::Device,
        memory_manager: &mut MemoryManager,
        transfer_command_buffer: vk::CommandBuffer,
        image_index: usize,
        render_stats: &mut RenderStats,
    ) -> bool {
        let start_time = Instant::now();
        let mut transferring = false;

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            logical_device
                .reset_command_buffer(transfer_command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset Transfer command buffer!");
            logical_device
                .begin_command_buffer(transfer_command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording of Transfer command buffer!");
        }

        self.buffer_objects
            .iter_mut()
            .filter(|bo| bo.is_dirty[image_index])
            .for_each(|bo| {
                let staging_buffer = bo.staging(image_index);
                let device_buffer = bo.device(image_index);

                unsafe {
                    let data_slice = from_raw_parts(bo.raw_array.start(), bo.raw_array.len_bytes());
                    if data_slice.is_empty() {
                        bo.is_dirty[image_index] = false;
                        return;
                    }
                    memory_manager.copy_to_buffer_memory(logical_device, staging_buffer, data_slice);
                }

                transferring = true;

                let copy_region = [vk::BufferCopy::builder()
                    .src_offset(0)
                    .dst_offset(0)
                    .size(bo.raw_array.len_bytes() as vk::DeviceSize)
                    .build()];
                unsafe {
                    logical_device.cmd_copy_buffer(
                        transfer_command_buffer,
                        staging_buffer,
                        device_buffer,
                        &copy_region,
                    );
                }

                bo.is_dirty[image_index] = false;
            });

        unsafe {
            logical_device
                .end_command_buffer(transfer_command_buffer)
                .expect("Failed to end recording of Transfer command buffer!");
        }

        render_stats.transfer_commands_bake_time = start_time.elapsed();
        transferring
    }

    pub fn handle_buffer_overflow(
        &mut self,
        device: &ash::Device,
        memory_manager: &mut MemoryManager,
        handle: BufferObjectHandle,
        image_count: usize,
    ) {
        debug_assert!(self.buffer_objects.len() > handle);
        log_debug_once!("buffer object overflow handle={}:", handle);
        self.buffer_objects[handle].handle_buffer_overflow(device, memory_manager, image_count);
    }

    pub fn destroy(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager) {
        for buffer in self.buffer_objects.iter_mut() {
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
    is_dirty: Vec<bool>,
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

        let dirty_array = vec![true; image_count];

        BufferObject {
            buffer_object_type,
            capacity_bytes,
            staging_buffer,
            device_buffer,
            raw_array: RawArray::new::<T>(capacity).unwrap(),
            assigned_pipelines: Vec::new(),
            is_growable,
            is_dirty: dirty_array,
        }
    }

    pub fn build(&mut self, device: &ash::Device, memory_manager: &mut MemoryManager, image_count: usize) {
        for _i in 0..image_count {
            let usage = match self.buffer_object_type {
                BufferObjectType::Uniform(_) => vk::BufferUsageFlags::UNIFORM_BUFFER,
                BufferObjectType::Storage => unimplemented!(),
                BufferObjectType::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
            };

            let staging_buf = memory_manager.create_staging_buffer(device, self.capacity_bytes as vk::DeviceSize);
            let device_buf = memory_manager.create_device_buffer(device, self.capacity_bytes as vk::DeviceSize, usage);

            self.staging_buffer.push(staging_buf);
            self.device_buffer.push(device_buf);
        }
        self.is_dirty = vec![true; image_count];
    }

    pub fn assign_pipeline(&mut self, pipeline_handle: PipelineHandle) {
        self.assigned_pipelines.push(pipeline_handle);
    }

    pub fn push<T>(&mut self, data: T) -> Result<RawArrayPtr, PushError> {
        self.is_dirty.fill(true);
        self.raw_array.push(data)
    }

    pub fn reset(&mut self) {
        self.raw_array.reset();
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

    pub fn devices(&self) -> &[vk::Buffer] {
        &self.device_buffer
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
        if !self.is_growable {
            return;
        }

        let new_cap = self.raw_array.len() * 2;
        self.raw_array.resize(new_cap).expect("Failed to resize dynamic buffer");
        self.capacity_bytes *= 2;

        log_debug!(" - resized device buffers to {}", self.capacity_bytes);
        log_debug!(" - {:?}", self.raw_array);

        unsafe {
            device.device_wait_idle().expect("Failed to wait device idle!");
            self.destroy(device, memory_manager);
            self.build(device, memory_manager, image_count);
        }
    }

    pub fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
}
