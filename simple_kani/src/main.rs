// mod aaa;
// mod backup;

use autokani::{kani_test, kani_arbitrary, extend_arbitrary};

#[kani_test]
pub fn u8_input(s: u8) {
    let _ = s;
}
#[kani_test]
pub fn i16_input(s: i16) {
    let _ = s;
}
#[kani_test]
pub fn f16_input(s: f32) {
    let _ = s;
}
#[kani_test]
pub fn bool_input(s: bool) {
    let _ = s;
}
fn main() {
    println!("Hello, world!");
}
