use std::alloc::{alloc, dealloc, Layout};

pub type RawArrayPtr = *const u8;

type RawArrayInternalPtr = *mut u8;

pub struct RawArray {
    buf_ptr: Option<RawArrayInternalPtr>,
    capacity: usize,

    data_size: usize,
    write_index: usize,
}

impl RawArray {
    pub fn new<T>(capacity: usize) -> RawArray {
        let size = std::mem::size_of::<T>();

        let buf = if capacity > 0 {
            let buf_ptr = unsafe {
                let layout = Layout::from_size_align_unchecked(capacity * size, 1);

                alloc(layout) as RawArrayInternalPtr
            };

            if buf_ptr.is_null() {
                panic!("Failed to allocate PushConstantBuffer");
            }
            Option::Some(buf_ptr)
        } else {
            Option::None
        };

        RawArray {
            buf_ptr: buf,
            capacity,
            data_size: size,
            write_index: 0,
        }
    }

    pub fn push<T>(&mut self, data: T) -> RawArrayPtr {
        debug_assert!(self.write_index < self.capacity);
        // TODO grow on overflow?

        unsafe {
            let ptr = self.buf_ptr.unwrap().add(self.write_index * self.data_size);
            std::ptr::copy_nonoverlapping(&data as *const T as *const u8, ptr, self.data_size);

            self.write_index += 1;

            ptr
        }
    }

    pub fn start(&self) -> RawArrayPtr {
        debug_assert!(self.buf_ptr.is_some());
        self.buf_ptr.unwrap()
    }

    pub fn data_size(&self) -> usize {
        self.data_size
    }

    pub fn len(&self) -> usize {
        self.write_index
    }

    pub fn reset(&mut self) {
        self.write_index = 0;
    }
}

impl Drop for RawArray {
    fn drop(&mut self) {
        if self.buf_ptr.is_some() {
            unsafe {
                dealloc(
                    self.buf_ptr.unwrap() as *mut u8,
                    Layout::from_size_align_unchecked(self.capacity * self.data_size, 1),
                )
            };
        }
    }
}
