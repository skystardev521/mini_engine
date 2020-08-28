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

//第1个轮子占用8位
const TVR_BITS: usize = 8;
//第2~5轮子各占用6位
const TVN_BITS: usize = 6;

//第1个轮子槽数量为256
const TVR_SIZE: usize = (1 << TVR_BITS);
//第2~5轮子槽数量为64
const TVN_SIZE: usize = (1 << TVN_BITS);

//第1个轮子槽0~255
const TVR_MSAK: usize = (TVR_SIZE - 1);

//第2~5轮子槽0~63
const TVN_MSAK: usize = (TVN_SIZE - 1);

macro_rules! tv_idx {
    ($tick:expr, $idx:expr) => {
        (($tick >> (TVR_BITS + ($idx) * TVN_BITS)) & TVN_MSAK)
    };
}

pub struct WTimer {
    wheel: Wheel,
    /// Tick间隔毫秒
    tick_size: u16,
}

impl Drop for WTimer {
    fn drop(&mut self) {}
}

struct Wheel {
    /*
    //当前刻度
    //uint64_t curTick;
    List tv2[TVN_SIZE];
    List tv3[TVN_SIZE];
    List tv4[TVN_SIZE];
    List tv5[TVN_SIZE];
    Timer* runningTimer;
    */
    cur_tick: u64,
    tv1: [VecDeque<TEntity>; TVR_SIZE],
    tv2: [VecDeque<TEntity>; TVN_SIZE],
    tv3: [VecDeque<TEntity>; TVN_SIZE],
    tv4: [VecDeque<TEntity>; TVN_SIZE],
    tv5: [VecDeque<TEntity>; TVN_SIZE],
}

impl Wheel {}

pub trait TimedTask {
    /// true:任务完成
    fn execute(&mut self) -> bool;
}

impl WTimer {
    pub fn scheduled(ts: u64) {}

    /*
    fn Cascade(List* tv, uint8_t idx){

    }
    */
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            vec_deque: VecDeque::new(),
        }
    }

    pub fn push_task(&mut self, delay: u64, interval: u64, task: Box<dyn TimedTask>) {
        let te = TEntity {
            task,
            interval,
            expire: time::timestamp() + delay,
        };
        self.vec_deque.push_back(te);
    }

    pub fn execute(&mut self) {
        while let Some(entity) = &mut self.vec_deque.pop_front() {
            entity.task.execute();
        }
    }
}

pub struct TestTimedTask {
    id: u16,
}

impl TimedTask for TestTimedTask {
    fn execute(&mut self) -> bool {
        self.id > 0
    }
}

use crate::wtimer;
#[test]
fn test_timer() {
    let v = tv_idx!(0, 0);
    let mut timer = wtimer::Timer::new();

    let task = Box::new(TestTimedTask { id: 1 });

    timer.push_task(1, 10, task);

    timer.execute();

    println!("test_timer finish");
}
