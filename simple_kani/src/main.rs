// mod aaa;
// mod backup;

use autokani::{kani_test, kani_arbitrary};

#[kani_arbitrary]
pub struct Array {
    data: Vec<i32>,
    len: usize,
    capacity: usize,
}

impl Array {
    pub fn new(cap: usize) -> Self {
        Array {
            data: Vec::with_capacity(cap),
            len: 0,
            capacity: cap,
        }
    }
    pub fn push(&mut self, val: i32) {
        if self.len == self.capacity {
            return;
        }
        self.data.push(val);
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<i32> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.data.pop()
    }
    /// # Safety
    /// This function is unsafe because it doesn't check if the index is out of bounds.
    pub unsafe fn get_unchecked(&self, index: usize) -> i32 {
        *self.data.get_unchecked(index)
    }
    // #[kani_test]
    pub fn get_unsound(&self, index: usize) -> Option<i32> {
        Some(unsafe { self.get_unchecked(index) })
    }
}
// #[kani_test]
pub fn get_unsound1(arr: &Array, index: usize) -> Option<i32> {
    Some(unsafe { arr.get_unchecked(index) })
}
// #[kani_test]
pub fn get_sound(arr: &Array, index: usize) -> Option<i32> {
    if index >= arr.len {
        return None;
    }
    Some(unsafe { arr.get_unchecked(index) })
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    pub fn test_get_unsound() {
        // let mut arr = Array::new(16);
        let arr = kani::any::<Array>();
        let index = kani::any::<usize>();
        let _ = get_unsound1(&arr, index);
    }

    // #[kani::proof]
    // pub fn test_ptr_input() {
    //     let mut generator = kani::PointerGenerator::<{std::mem::size_of::<u32>()}>::new();
    //     let ptr2: *const u32 = generator.any_alloc_status().ptr;
    //     let _ = ptr_input(ptr2);
    // }
}

fn main() {
    println!("Hello, world!");
}
