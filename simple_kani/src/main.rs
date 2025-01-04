// mod aaa;
// mod backup;

use autokani::kani_test;




// #[kani_test]
// pub fn multi_param1(a: (i16, u8), b: f32, v: Vec<String>) {
//     let y = a.1 + b as u8;
//     let _ = v[y as usize];
// }

// #[kani_test]
// pub fn digit_to_char(digit: u32) -> u8 {
//     const TABLE: [u8; 10] = [
//         b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'
//     ];
//     unsafe { *TABLE.get_unchecked(digit as usize) }
// }
// #[kani_test]
// pub fn digit_to_char1(digit: &u32) -> u8 {
//     const TABLE: [u8; 10] = [
//         b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'
//     ];
//     // *digit %= 100;
//     unsafe { *TABLE.get_unchecked(*digit as usize) }
// }

#[cfg(kani)]
mod verification {
    use super::*;

    // #[kani::proof]
    // pub fn test_slice_input() {
    //     let mut stream_obj = kani::any::<[i32; 16usize]>();
    //     let mut stream = kani::slice::any_slice_of_array_mut(&mut stream_obj);
    //     let _ = slice_input2(stream);
    // }

    // #[kani::proof]
    // pub fn test_slice_input2() {
    //     let mut stream_obj = kani::vec::any_vec::<i32, 16usize>();
    //     let mut slice = stream_obj.as_mut_slice();
    //     let mut stream = kani::slice::any_slice_of_array_mut(&mut slice);
    //     let _ = slice_input2(stream);
    // }

    // #[kani::proof]
    // pub fn test_u8_input() {
    //     let mut s_obj: u8 = kani::any();
    //     let mut s = &mut s_obj;
    //     let _ = u8_input(s);
    // }
    // #[kani::proof]
    // fn test_char_to_str() {
    //     const LEN: usize = 10;
    //     let stream = kani::any::<[u8; LEN]>();
    //     let _ = char_to_str(&stream);
    // }

    // #[kani::proof]
    // fn test_digit_to_char() {
    //     let digit = kani::any::<u32>();
    //     kani::assume(digit < 100000000);
    //     let _ = digit_to_char(digit);
    // }

    // #[kani::proof]
    // fn test_digit_to_char1() {
    //     let digit_obj = kani::any();
    //     kani::assume(digit_obj < 1000000);
    //     let digit = &digit_obj;
    //     let _ = digit_to_char1(digit);
    // }

    // #[kani::proof]
    // #[kani::unwind(100)]
    // fn test_str_input() {
    //     let char_arr = kani::any::<[char; 10]>();
    //     let s = String::from_iter(char_arr);
    //     let _ = str_input(s);
    // }
}

fn main() {
    println!("Hello, world!");
}
