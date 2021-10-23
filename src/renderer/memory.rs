use std::ptr;

use ash::vk;

use crate::renderer::pipeline::{Index, VertexInputDescription};
use ash::vk::PhysicalDeviceMemoryProperties;
use std::collections::HashMap;

//  TODO: make it possible allocate a buffer on preexisting memory.
//  TODO: Maybe create an allocator that can handle the memory allocation completety separate from buffer allocation
pub struct MemoryManager {
    physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    buffer_to_chunk_map: HashMap<vk::Buffer, vk::DeviceMemory>,
}

impl MemoryManager {}

impl MemoryManager {
    pub fn new(physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties) -> MemoryManager {
        MemoryManager {
            physical_device_memory_properties,
            buffer_to_chunk_map: HashMap::new(),
        }
    }

    pub fn create_static_vertex_buffer_sync<T: VertexInputDescription>(
        &mut self,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        vertices: &[T],
    ) -> vk::Buffer {
        let (buffer, device_memory) = create_device_local_buffer_sync(
            device,
            &self.physical_device_memory_properties,
            command_pool,
            submit_queue,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vertices,
        );

        self.buffer_to_chunk_map.insert(buffer, device_memory);

        buffer
    }

    pub fn create_index_buffer(
        &mut self,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        indicies: &[Index],
    ) -> vk::Buffer {
        let (buffer, device_memory) = create_device_local_buffer_sync(
            device,
            &self.physical_device_memory_properties,
            command_pool,
            submit_queue,
            vk::BufferUsageFlags::INDEX_BUFFER,
            indicies,
        );

        self.buffer_to_chunk_map.insert(buffer, device_memory);

        buffer
    }

    pub fn create_uniform_buffers(
        &mut self,
        logical_device: &ash::Device,
        buffer_size: usize,
        swapchain_image_count: usize,
    ) -> Vec<vk::Buffer> {
        let mut uniform_buffers = Vec::with_capacity(swapchain_image_count);

        for _ in 0..swapchain_image_count {
            let (uniform_buffer, uniform_buffer_memory) = _create_buffer(
                logical_device,
                buffer_size as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                &self.physical_device_memory_properties,
            );
            uniform_buffers.push(uniform_buffer);
            self.buffer_to_chunk_map.insert(uniform_buffer, uniform_buffer_memory);
        }

        uniform_buffers
    }

    pub fn create_staging_buffer(&mut self, logical_device: &ash::Device, buffer_size: vk::DeviceSize) -> vk::Buffer {
        let (staging_buffer, staging_buffer_memory) = _create_buffer(
            logical_device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &self.physical_device_memory_properties,
        );

        self.buffer_to_chunk_map.insert(staging_buffer, staging_buffer_memory);

        staging_buffer
    }

    pub fn create_device_buffer(&mut self, logical_device: &ash::Device, buffer_size: vk::DeviceSize) -> vk::Buffer {
        let (staging_buffer, staging_buffer_memory) = _create_buffer(
            logical_device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &self.physical_device_memory_properties,
        );

        self.buffer_to_chunk_map.insert(staging_buffer, staging_buffer_memory);

        staging_buffer
    }

    pub unsafe fn copy_to_buffer_memory<T>(&mut self, logical_device: &ash::Device, buffer: vk::Buffer, data: &[T]) {
        let memory = self.get_device_memory(buffer);
        _copy_to_device_memory(logical_device, data, memory);
    }

    pub unsafe fn destroy_buffer(&mut self, logical_device: &ash::Device, buffer: vk::Buffer) {
        let memory_option = self.buffer_to_chunk_map.remove(&buffer);

        if let Some(memory) = memory_option {
            logical_device.destroy_buffer(buffer, None);
            logical_device.free_memory(memory, None);
        }
    }

    pub unsafe fn destroy(&mut self, logical_device: &ash::Device) {
        for (buffer, memory) in self.buffer_to_chunk_map.iter() {
            logical_device.destroy_buffer(*buffer, None);
            logical_device.free_memory(*memory, None);
        }
    }

    pub fn physical_device_memory_properties(&self) -> &PhysicalDeviceMemoryProperties {
        &self.physical_device_memory_properties
    }

    pub fn get_device_memory(&self, buffer: vk::Buffer) -> vk::DeviceMemory {
        *self.buffer_to_chunk_map.get(&buffer).expect("Unknown buffer memory!")
    }
}

fn create_device_local_buffer_sync<T>(
    device: &ash::Device,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size = std::mem::size_of_val(data) as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = _create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        device_memory_properties,
    );

    unsafe {
        _copy_to_device_memory(device, data, staging_buffer_memory);
    }

    let (device_local_buffer, device_local_buffer_memory) = _create_buffer(
        device,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | usage,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        device_memory_properties,
    );

    _copy_buffer_device_blocking(
        device,
        submit_queue,
        command_pool,
        staging_buffer,
        device_local_buffer,
        buffer_size,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    (device_local_buffer, device_local_buffer_memory)
}

fn _create_buffer(
    device: &ash::Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    required_memory_properties: vk::MemoryPropertyFlags,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_create_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };

    let buffer = unsafe {
        device
            .create_buffer(&buffer_create_info, None)
            .expect("Failed to create Vertex Buffer")
    };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memory_type = find_memory_type(
        mem_requirements.memory_type_bits,
        required_memory_properties,
        device_memory_properties,
    );

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
    };

    let buffer_memory = unsafe {
        device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory!")
    };

    unsafe {
        device
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .expect("Failed to bind Buffer");
    }

    (buffer, buffer_memory)
}

fn find_memory_type(
    type_filter: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: &vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties) {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type!")
}

unsafe fn _copy_to_device_memory<T>(device: &ash::Device, data: &[T], dst_memory: vk::DeviceMemory) {
    let buffer_size = std::mem::size_of_val(data) as vk::DeviceSize;

    // TODO the size should be checked here. Need to track the capcity of the memory.

    let data_ptr = device
        .map_memory(dst_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
        .expect("Failed to Map Memory") as *mut T;

    data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

    device.unmap_memory(dst_memory);
}

fn _copy_buffer_device_blocking(
    device: &ash::Device,
    submit_queue: vk::Queue,
    command_pool: vk::CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: 1,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .expect("Failed to allocate Command Buffer")
    };
    let command_buffer = command_buffers[0];

    let begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next: ptr::null(),
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        p_inheritance_info: ptr::null(),
    };

    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Failed to begin Command Buffer");

        let copy_regions = [vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];

        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);

        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end Command Buffer");
    }

    let submit_info = [vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: ptr::null(),
        wait_semaphore_count: 0,
        p_wait_semaphores: ptr::null(),
        p_wait_dst_stage_mask: ptr::null(),
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        signal_semaphore_count: 0,
        p_signal_semaphores: ptr::null(),
    }];

    unsafe {
        device
            .queue_submit(submit_queue, &submit_info, vk::Fence::null())
            .expect("Failed to Submit Queue.");
        device.queue_wait_idle(submit_queue).expect("Failed to wait Queue idle");

        device.free_command_buffers(command_pool, &command_buffers);
    }
}
