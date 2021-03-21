use crate::renderer::constants::MAX_FRAMES_IN_FLIGHT;
use crate::renderer::context::PipelineHandle;
use crate::renderer::memory::MemoryManager;
use crate::renderer::pipeline::UniformData;
use ash::version::DeviceV1_0;
use ash::vk;

#[derive(Clone, Debug, Copy)]
pub enum UniformStage {
    Vertex,
    Fragment,
}

pub(super) struct Uniform {
    stage: UniformStage,
    data: Vec<u8>,
    assigned_pipelines: Vec<PipelineHandle>,

    dirty_uniform: Vec<bool>,
    buffers: Vec<vk::Buffer>,
    memory: Vec<vk::DeviceMemory>,

    is_built: bool,
}

impl Uniform {
    pub(super) fn new(size: usize, stage: UniformStage) -> Self {
        Uniform {
            stage,
            data: Vec::with_capacity(size),
            assigned_pipelines: Vec::new(),
            dirty_uniform: Vec::with_capacity(MAX_FRAMES_IN_FLIGHT),
            buffers: Vec::with_capacity(MAX_FRAMES_IN_FLIGHT),
            memory: Vec::with_capacity(MAX_FRAMES_IN_FLIGHT),
            is_built: false,
        }
    }

    pub(super) fn assign_pipeline(&mut self, pipeline_handle: PipelineHandle) {
        self.assigned_pipelines.push(pipeline_handle);
    }

    pub(super) fn assigned_pipelines(&self) -> &[PipelineHandle] {
        &self.assigned_pipelines
    }

    pub(super) fn set_data<T: UniformData>(&mut self, data: T) {
        unsafe {
            let dst_ptr = self.data.as_mut_ptr() as *mut T;
            dst_ptr.copy_from_nonoverlapping(&data as *const T, 1);
        }

        for i in 0..self.dirty_uniform.len() {
            self.dirty_uniform[i] = true;
        }
    }

    pub(super) fn update_device_memory(&mut self, logical_device: &ash::Device, image_index: usize) {
        if self.dirty_uniform[image_index] {
            let buffer_size = self.data.capacity() as u64;
            let memory = self.memory[image_index];
            unsafe {
                let data_ptr = logical_device
                    .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                    .expect("Failed to Map Memory") as *mut u8;

                data_ptr.copy_from_nonoverlapping(self.data.as_ptr(), buffer_size as usize);

                logical_device.unmap_memory(memory);
            }

            self.dirty_uniform[image_index] = false;
        }
    }

    pub(super) fn build(
        &mut self,
        logical_device: &ash::Device,
        memory_manager: &mut MemoryManager,
        image_count: usize,
    ) {
        if self.is_built {
            panic!("Uniform already built!");
        }

        self.buffers = memory_manager.create_uniform_buffers(logical_device, self.size(), image_count);
        self.memory = self
            .buffers
            .iter()
            .map(|buf| memory_manager.get_device_memory(*buf))
            .collect();

        self.dirty_uniform.resize(image_count, true);

        assert_eq!(self.buffers.len(), self.memory.len());
        assert_eq!(self.buffers.len(), self.dirty_uniform.len());
    }

    pub(super) fn buffers(&self) -> &[vk::Buffer] {
        &self.buffers
    }

    pub(super) fn stage(&self) -> UniformStage {
        self.stage
    }

    pub(super) fn destroy(&mut self, logical_device: &ash::Device, memory_manager: &mut MemoryManager) {
        for buffer in self.buffers.iter() {
            unsafe {
                memory_manager.destroy_buffer(logical_device, *buffer);
            }
        }
        self.buffers.clear();
        self.memory.clear();
        self.dirty_uniform.clear();

        self.is_built = false;
    }

    pub(super) fn size(&self) -> usize {
        self.data.capacity()
    }
}
