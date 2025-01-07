// mod aaa;
// mod backup;

use autokani::{extend_arbitrary, kani_arbitrary, autokani_func, autokani_method};
pub struct Array {
    data: Vec<i32>,
    len: usize,
    capacity: usize,
}
#[extend_arbitrary]
impl Array {
    // #[autokani_method]
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
    #[autokani_func]
    pub fn get_unsound(&self, index: usize) -> Option<i32> {
        Some(unsafe { self.get_unchecked(index) })
    }
    #[autokani_func]
    pub fn get_sound(&self, index: usize) -> Option<i32> {
        if index >= self.len {
            return None;
        }
        Some(unsafe { self.get_unchecked(index) })
    }
}
fn main() {
    println!("Hello, world!");
}
