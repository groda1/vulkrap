use std::alloc::{alloc, dealloc, Layout};

pub type PushConstantPtr = *const u8;

type PushConstantInternal = *mut u8;

pub struct PushConstantBuffer {
    buf_ptr: Option<PushConstantInternal>,
    len: usize,

    data_size: usize,
    write_index: usize,
}

impl PushConstantBuffer {
    pub fn new<T>(capacity: usize) -> PushConstantBuffer {
        let size = std::mem::size_of::<T>();

        let buf = if capacity > 0 {
            let buf_ptr = unsafe {
                let layout = Layout::from_size_align_unchecked(capacity * size, 1);

                alloc(layout) as PushConstantInternal
            };

            if buf_ptr.is_null() {
                panic!("Failed to allocate PushConstantBuffer");
            }
            Option::Some(buf_ptr)
        } else {
            Option::None
        };

        PushConstantBuffer {
            buf_ptr: buf,
            len: capacity,
            data_size: size,
            write_index: 0,
        }
    }

    pub fn push<T>(&mut self, data: T) -> PushConstantPtr {
        debug_assert!(self.write_index < self.len);

        unsafe {
            let ptr = self.buf_ptr.unwrap().add(self.write_index * self.data_size);
            std::ptr::copy_nonoverlapping(&data as *const T as *const u8, ptr, self.data_size);

            self.write_index += 1;

            ptr
        }
    }

    pub fn data_size(&self) -> usize {
        self.data_size
    }

    pub fn reset(&mut self) {
        self.write_index = 0;
    }
}

impl Drop for PushConstantBuffer {
    fn drop(&mut self) {
        if self.buf_ptr.is_some() {
            unsafe {
                dealloc(
                    self.buf_ptr.unwrap() as *mut u8,
                    Layout::from_size_align_unchecked(self.len, self.data_size),
                )
            };
        }
    }
}
