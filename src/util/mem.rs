use std::alloc::{alloc, Layout};
pub fn raw_alloc<T>() -> Box<T> {
    let layout = Layout::new::<T>();
    let mut buf = unsafe {
        let ptr = alloc(layout) as *mut T;
        Box::from_raw(ptr)
    };

    buf
}

#[cfg(test)]
mod tests {
    use crate::util::mem::raw_alloc;
    use stopwatch::Stopwatch;

    const SIZE: usize = 5_000_000;

    #[test]
    fn into_slice() {
        let sw = Stopwatch::start_new();
        let mut mem = vec![1 as u32; SIZE].into_boxed_slice();
        let sum: u32 = mem.iter().sum();
        println!("sum: {}", sum);
        println!("Took: {} ms", sw.elapsed_ms());
    }

    #[test]
    fn allocing() {
        let sw = Stopwatch::start_new();
        let mut mem = raw_alloc::<[u32; SIZE]>();
        mem.iter_mut().for_each(|v| *v = 1);
        let sum: u32 = mem.iter().sum();
        println!("sum: {}", sum);
        println!("Took: {} ms", sw.elapsed_ms());
    }
}
