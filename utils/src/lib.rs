pub mod bytes;
pub mod capi;
pub mod logger;
pub mod time;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
