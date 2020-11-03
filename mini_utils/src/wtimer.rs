use crate::time;
use std::collections::VecDeque;
use std::mem::{self, MaybeUninit};

//第1个轮子占用8位
const TVR_BITS: u64 = 8;
//第2~5轮子各占用6位
const TVN_BITS: u64 = 6;

/// 第1个轮子槽数量为256
const TVR_SIZE: u64 = 1 << TVR_BITS;

/// 第2~5轮子槽数量为64
const TVN_SIZE: u64 = 1 << TVN_BITS;

/// 第1个轮子槽0~255
const TVR_MSAK: u64 = TVR_SIZE - 1;

/// 第2~5轮子槽0~63
const TVN_MSAK: u64 = TVN_SIZE - 1;

macro_rules! tv_idx {
    ($tick:expr, $idx:expr) => {
        (($tick >> (TVR_BITS + ($idx) * TVN_BITS)) & TVN_MSAK) as usize
    };
}

macro_rules! cascade {
    ($wheel:expr, $tv:ident, $idx:expr) => {{
        let mut deque = mem::replace(&mut $wheel.$tv[$idx], VecDeque::new());
        while let Some(entity) = deque.pop_front() {
            push_entity($wheel, entity);
        }
        $idx > 0
    }};
}

pub struct WTimer {
    wheel: Wheel,
}
pub trait IWTask {
    /// return true:任务完成
    fn execute(&mut self) -> bool;
}

pub struct TEntity {
    /// 过期时间
    pub(crate) expire: u64,
    /// 运行间隔
    pub(crate) interval: u64,
    /// 任务结构体
    pub(crate) task: Box<dyn IWTask>,
}

struct Wheel {
    cur_tick: u64,
    /// Tick间隔毫秒
    tick_size: u64,
    //cur_entity:TEntity,
    tv1: [VecDeque<TEntity>; TVR_SIZE as usize],
    tv2: [VecDeque<TEntity>; TVN_SIZE as usize],
    tv3: [VecDeque<TEntity>; TVN_SIZE as usize],
    tv4: [VecDeque<TEntity>; TVN_SIZE as usize],
    tv5: [VecDeque<TEntity>; TVN_SIZE as usize],
}

impl WTimer {
    /// tick_size: tick间隔 单位(毫秒)
    pub fn new(tick_size: u16) -> Self {
        let tick_size: u64 = if tick_size < 1 { 1 } else { tick_size as u64 };
        let cur_tick = time::timestamp() / tick_size;

        let mut wheel = Wheel {
            cur_tick,
            tick_size,
            tv1: unsafe { MaybeUninit::uninit().assume_init() },
            tv2: unsafe { MaybeUninit::uninit().assume_init() },
            tv3: unsafe { MaybeUninit::uninit().assume_init() },
            tv4: unsafe { MaybeUninit::uninit().assume_init() },
            tv5: unsafe { MaybeUninit::uninit().assume_init() },
        };

        for i in 0..TVR_SIZE as usize {
            wheel.tv1[i] = VecDeque::new();
        }

        for i in 0..TVN_SIZE as usize {
            wheel.tv2[i] = VecDeque::new();
            wheel.tv3[i] = VecDeque::new();
            wheel.tv4[i] = VecDeque::new();
            wheel.tv5[i] = VecDeque::new();
        }

        WTimer { wheel }
    }
    pub fn scheduled(&mut self, timestamp: u64) {
        let wheel = &mut self.wheel;
        let mut w_tick = wheel.cur_tick;
        let cur_tick = timestamp / wheel.tick_size;
        while w_tick < cur_tick {
            let idx = w_tick & TVR_MSAK;
            if idx != 0
                && !cascade!(wheel, tv2, tv_idx!(w_tick, 0))
                && !cascade!(wheel, tv3, tv_idx!(w_tick, 1))
                && !cascade!(wheel, tv4, tv_idx!(w_tick, 2))
            {
                cascade!(wheel, tv5, tv_idx!(w_tick, 3));
            }

            w_tick += 1;
            wheel.cur_tick = w_tick;
            let mut deque = mem::replace(&mut wheel.tv1[idx as usize], VecDeque::new());
            while let Some(mut entity) = deque.pop_front() {
                if entity.task.execute() {
                    continue;
                }
                entity.expire = timestamp + entity.interval;
                push_entity(wheel, entity);
            }
        }
    }

    pub fn push_task(&mut self, delay: u64, interval: u64, task: Box<dyn IWTask>) {
        let interval = if interval < self.wheel.tick_size {
            self.wheel.tick_size
        } else {
            let n = interval % self.wheel.tick_size;
            if n == 0 {
                interval
            } else {
                interval + (self.wheel.tick_size - n)
            }
        };

        push_entity(
            &mut self.wheel,
            TEntity {
                task,
                interval,
                expire: time::timestamp() + delay,
            },
        );
    }
}

fn push_entity(wheel: &mut Wheel, entity: TEntity) {
    let mut ticks = entity.expire / wheel.tick_size;
    let mut idx = (ticks - wheel.cur_tick) as u64;

    let entity_deque: &mut VecDeque<TEntity>;
    if idx < TVR_SIZE {
        let i = ticks & TVR_MSAK;
        entity_deque = &mut wheel.tv1[i as usize];
    } else if idx < 1 << (TVR_BITS + TVN_BITS) {
        let i = (ticks >> TVR_BITS) & TVN_MSAK;
        entity_deque = &mut wheel.tv2[i as usize];
    } else if idx < 1 << (TVR_BITS + 2 * TVN_BITS) {
        let i = (ticks >> (TVR_BITS + TVN_BITS)) & TVN_MSAK;
        entity_deque = &mut wheel.tv3[i as usize];
    } else if idx < 1 << (TVR_BITS + 3 * TVN_BITS) {
        let i = (ticks >> (TVR_BITS + 2 * TVN_BITS)) & TVN_MSAK;
        entity_deque = &mut wheel.tv4[i as usize];
    }
    /*else if idx < 0 {
        entity_deque = &mut self.wheel.tv1[(self.wheel.cur_tick & TVR_MSAK) as usize];

    }*/
    else {
        if idx > 0xffffffffu64 {
            idx = 0xffffffffu64;
            ticks = idx + wheel.cur_tick;
        }
        let i = (ticks >> (TVR_BITS + 3 * TVN_BITS)) & TVN_MSAK;
        entity_deque = &mut wheel.tv5[i as usize];
    }
    entity_deque.push_back(entity);
}

pub struct TestIWTask {
    pub id: u64,
    pub name: String,
}

impl IWTask for TestIWTask {
    fn execute(&mut self) -> bool {
        self.id += 1;
        if self.id == 1 || self.id % 600 == 0 {
            println!(
                "time:{} id:{} name:{}",
                time::timestamp(),
                self.id,
                self.name
            );
        }
        self.id > 9999999999999
    }
}
#[cfg(test)]
mod test {
    use crate::time;
    use crate::wtimer::TestIWTask;
    use crate::wtimer::WTimer;

    #[test]
    fn test_timer() {
        let mut wtimer = WTimer::new(1);

        for i in 0..9 {
            let task = Box::new(TestIWTask {
                id: 0,
                name: format!("name:{}", i),
            });
            wtimer.push_task(1, 10, task);
        }

        wtimer.scheduled(time::timestamp());

        println!("test_timer finish");
    }
}
