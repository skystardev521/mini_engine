pub mod config;
pub mod service;
pub mod task;

#[macro_use]
pub mod result;

pub(crate) mod connect;
pub(crate) mod task_mgmt;
pub(crate) mod thread_pool;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
