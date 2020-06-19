pub mod bytes;
pub mod ffi_ext;
pub mod logger;
pub mod time_ext;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
