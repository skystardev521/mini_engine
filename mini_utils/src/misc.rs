pub fn as_mut(t: &T)->&mut T{
    unsafe {&mut * (t as *const T as * mut T)}
}