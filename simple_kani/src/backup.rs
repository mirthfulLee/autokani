//// Basic Types //////////////////////////////////////////////
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

//// String //////////////////////////////////////////////
#[kani_test]
pub fn string_input(s: String) {
    let _ = s;
}

//// str //////////////////////////////////////////////
#[kani_test]
pub fn str_input(s: &str) {
    let _ = s;
}

//// multiple params //////////////////////////////////////////////
#[kani_test]
pub fn multi_param(a: i16, b: u8, c: f32, d: bool) {
    let _ = a;
    let x = b as f32 + c ;
    if d {
        let y = b + c as u8;
    }
}

//// slice //////////////////////////////////////////////
#[kani_test]
pub fn char_to_str(stream: &[u8]) -> String {
    unsafe { String::from_utf8_unchecked(stream.to_vec()) }
}

#[kani_test]
pub fn slice_input(stream: [u8; 10]) -> String {
    unsafe { String::from_utf8_unchecked(stream.to_vec()) }
}

#[kani_test]
pub fn slice_input2(stream: &[i32; 10]) {
    let _ = stream;
}

#[kani_test]
pub fn slice_input2(stream: &mut [i32]) {
    let _ = stream;
}

#[kani_test]
pub fn slice_input3(stream: &[i32]) {
    let _ = stream;
}

//// Tuple //////////////////////////////////////////////

#[kani_test]
pub fn multi_param(a: (i16, u8), c: f32, d: bool) {
    let _ = a;
    let x = a.0 as f32 + c ;
    if d {
        let y = a.1 + c as u8;
    }
}

#[kani_test]
pub fn multi_param1(a: (i16, u8), c: f32, v: &[u8]) {
    let _ = a;
    let x = a.0 as f32 + c ;
    let y = a.1 + c as u8;
    let _ = v[y as usize];
}


//// Vec //////////////////////////////////////////////

#[kani_test]
pub fn to_str(stream: Vec<u8>) -> String {
    unsafe { String::from_utf8_unchecked(stream) }
}

#[kani_test]
pub fn multi_param1(a: (i16, u8), b: f32, v: Vec<i32>) {
    let y = a.1 + b as u8;
    let _ = v[y as usize];
}

//// Reference //////////////////////////////////////////////
#[kani_test]
pub fn to_str(stream: &Vec<i16>) {
    let _ = stream.clone();
}

#[kani_test]
pub fn multi_param1(a: (i16, u8), b: &f32, v: Vec<i32>) {
    let y = a.1 + *b as u8;
    let _ = v[y as usize];
}

#[kani_test]
pub fn char_to_str2(stream: (u32, char), a: &f32) {
    let _ = a;
}

//// Mutability //////////////////////////////////////////////

#[kani_test]
pub fn u8_input(s:&mut u8) {
    let _ = s;
}

#[kani_test]
pub fn multi_param_mut(a: (i16, u8), b: &mut f32, v: Vec<i32>) {
    *b += 1.0;
    let y = a.1 + *b as u8;
    let _ = v[y as usize];
}

#[kani_test]
pub fn multi_param_mut1(a: (i16, u8), b: &mut f32, mut v: Vec<i32>) {
    *b += 1.0;
    let y = a.1 + *b as u8;
    v[y as usize] = 0;
}

#[kani_test]
pub fn initialize_prefix(length: usize, buffer: &mut [u8]) {
    // Let's just ignore invalid calls
    if length > buffer.len() {
        return;
    }

    for i in 0..=length {
        buffer[i] = 0;
    }
}

//// Raw Pointer //////////////////////////////////////////////

#[kani_test]
pub fn ptr_input(s: *const u32) {
    let _ = s;
}
#[kani_test]
pub fn ptr_input2(s: *mut u32, i:u32) {
    let _ = s;
    let _ = unsafe { *s = i };
}


//// Structs //////////////////////////////////////////////

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
}
pub fn get_unsound(arr: &Array, index: usize) -> Option<i32> {
    Some(unsafe { arr.get_unchecked(index) })
}
pub fn get_sound(arr: &Array, index: usize) -> Option<i32> {
    if index >= arr.len {
        return None;
    }
    Some(unsafe { arr.get_unchecked(index) })
}