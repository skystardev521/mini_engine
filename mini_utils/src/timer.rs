use crate::time;
use std::collections::VecDeque;
pub struct TEntity {
    /// 过期时间
    pub(crate) expire: u64,
    /// 运行间隔
    pub(crate) interval: u64,
    /// 任务结构体
    pub(crate) task: Box<dyn TimedTask>,
}

pub struct Timer {
    pub vec_deque: VecDeque<TEntity>,
}

pub trait TimedTask {
    /// true:任务完成
    fn execute(&mut self) -> bool;
}

impl Timer {
    pub fn add_task(&mut self, delay: u64, interval: u64, task: Box<dyn TimedTask>) {
        let te = TEntity {
            task,
            interval,
            expire: time::timestamp() + delay,
        };
        self.vec_deque.push_back(te);
    }
}
