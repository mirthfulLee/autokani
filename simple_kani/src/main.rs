// mod aaa;
// mod backup;

use autokani::{kani_test, kani_arbitrary, extend_arbitrary};

#[cfg(kani)]
mod verification {
    use super::*;

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
