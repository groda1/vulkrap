use std::alloc::{alloc, dealloc, Layout};
use std::fmt::Debug;
use std::slice;

pub type RawArrayPtr = *const u8;

type RawArrayInternalPtr = *mut u8;

#[derive(Debug)]
pub struct RawArray {
    buf_ptr: RawArrayInternalPtr,
    capacity: usize,

    data_size: usize,
    write_index: usize,
}

impl RawArray {
    pub fn new<T>(capacity: usize) -> Result<RawArray, &'static str> {
        let size = std::mem::size_of::<T>();

        if capacity == 0 {
            return Err("Empty capacity");
        }

        let buf_ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(capacity * size, 1);
            alloc(layout) as RawArrayInternalPtr
        };

        if buf_ptr.is_null() {
            return Err("Failed to allocate memory");
        }

        Ok(RawArray {
            buf_ptr,
            capacity,
            data_size: size,
            write_index: 0,
        })
    }

    pub fn push<T>(&mut self, data: T) -> Result<RawArrayPtr, PushError> {
        if self.write_index >= self.capacity {
            return Err(PushError::Overflow);
        }

        unsafe {
            let ptr = self.buf_ptr.add(self.write_index * self.data_size);
            std::ptr::copy_nonoverlapping(&data as *const T as *const u8, ptr, self.data_size);

            self.write_index += 1;

            Ok(ptr)
        }
    }

    pub fn start(&self) -> RawArrayPtr {
        self.buf_ptr
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

    pub fn resize(&mut self, new_size: usize) -> Result<(), &'static str> {
        if new_size == 0 {
            return Err("Empty new size");
        }

        let new_ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(new_size * self.data_size, 1);
            alloc(layout) as RawArrayInternalPtr
        };

        if new_ptr.is_null() {
            return Err("Failed to allocate memory");
        }

        let copy_count = (self.capacity * self.data_size()).min(new_size * self.data_size);
        unsafe {
            std::ptr::copy_nonoverlapping(self.buf_ptr, new_ptr, copy_count);
        }

        unsafe {
            dealloc(
                self.buf_ptr,
                Layout::from_size_align_unchecked(self.capacity * self.data_size, 1),
            )
        };

        self.capacity = new_size;
        self.buf_ptr = new_ptr;
        self.write_index = copy_count / self.data_size;

        Ok(())
    }

    pub unsafe fn slice<T>(&self) -> &[T] {
        debug_assert!(std::mem::size_of::<T>() == self.data_size);
        slice::from_raw_parts(self.buf_ptr as *const T, self.write_index)
    }
}

impl Drop for RawArray {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.buf_ptr,
                Layout::from_size_align_unchecked(self.capacity * self.data_size, 1),
            )
        };
    }
}

pub enum PushError {
    Overflow,
}

#[cfg(test)]
mod tests {
    use crate::renderer::rawarray::RawArray;

    #[test]
    fn test_insert() {
        let mut raw_array = RawArray::new::<u32>(5).unwrap();
        println!("{:?}", raw_array);

        let slice = unsafe { raw_array.slice::<u32>() };
        assert_eq!(slice.len(), 0);
        println!("{:?}", slice);

        raw_array.push(1);
        raw_array.push(2);
        raw_array.push(3);

        let slice = unsafe { raw_array.slice::<u32>() };
        assert_eq!(slice.len(), 3);
        assert_eq!(slice, &[1, 2, 3]);
        println!("{:?}", slice);
    }

    #[test]
    fn test_resize_smaller() {
        let mut raw_array = RawArray::new::<u32>(5).unwrap();
        println!("{:?}", raw_array);

        let slice = unsafe { raw_array.slice::<u32>() };
        assert_eq!(slice.len(), 0);
        println!("{:?}", slice);

        raw_array.push(1);
        raw_array.push(2);
        raw_array.push(3);

        raw_array.resize(2);

        let slice = unsafe { raw_array.slice::<u32>() };
        assert_eq!(slice.len(), 2);
        assert_eq!(slice, &[1, 2]);
        println!("{:?}", slice);
    }

    #[test]
    fn test_resize_larger() {
        let mut raw_array = RawArray::new::<u32>(3).unwrap();
        println!("{:?}", raw_array);

        let slice = unsafe { raw_array.slice::<u32>() };
        assert_eq!(slice.len(), 0);
        println!("{:?}", slice);

        raw_array.push(1);
        raw_array.push(2);
        raw_array.push(3);

        raw_array.resize(6);
        raw_array.push(4);

        let slice = unsafe { raw_array.slice::<u32>() };
        assert_eq!(slice.len(), 4);
        assert_eq!(slice, &[1, 2, 3, 4]);
        println!("{:?}", slice);
    }
}
