pub mod config;
pub mod service;
pub mod task;

#[macro_use]
pub mod result;

pub(crate) mod connect;
pub(crate) mod execute;
pub(crate) mod workers;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
