use crate::sql_task::SqlTaskEnum;
use log::{error, warn};
use mini_utils::worker::RecvResEnum;
use mini_utils::worker::SendResEnum;
use mini_utils::worker::Worker;

pub struct Workers {
    poll_idx: usize,
    single_max_task_num: usize,
    vec_worker: Vec<Worker<SqlTaskEnum, ()>>,
}

impl Drop for Workers {
    fn drop(&mut self) {}
}

impl Workers {
    pub fn new(size: u8, single_max_task_num: u16) -> Self {
        Workers {
            poll_idx: 0,
            single_max_task_num: single_max_task_num as usize,
            vec_worker: Vec::with_capacity(size as usize),
        }
    }
    #[inline]
    fn next_poll_idx(&mut self) {
        self.poll_idx += 1;
        if self.poll_idx >= self.vec_worker.len() {
            self.poll_idx = 0;
        }
    }

    #[inline]
    pub fn push(&mut self, worker: Worker<SqlTaskEnum, ()>) {
        self.vec_worker.push(worker);
    }

    pub fn sender(&mut self, task_enum: SqlTaskEnum) -> bool {
        if self.vec_worker.is_empty() {
            return false;
        }

        if self.poll_idx >= self.vec_worker.len() {
            self.poll_idx = 0;
        }

        let mut send_result = false;
        let init_idx = self.poll_idx;
        let mut mut_task = task_enum;
        loop {
            let worker = &self.vec_worker[self.poll_idx];
            match worker.sender(mut_task) {
                SendResEnum::Success => {
                    send_result = true;
                    self.next_poll_idx();
                    break;
                }
                SendResEnum::Full(res_task) => {
                    mut_task = res_task;
                    warn!("Worker:{} Full", worker.get_name());
                    self.next_poll_idx();
                    if self.poll_idx == init_idx {
                        break;
                    }
                }
                SendResEnum::Disconnected(res_task) => {
                    mut_task = res_task;
                    error!("Worker:{} Disconnected", worker.get_name());
                    self.vec_worker.remove(self.poll_idx);
                    self.next_poll_idx();
                    if self.poll_idx == init_idx {
                        break;
                    }
                }
            }
        }
        send_result
    }

    pub fn receiver(&mut self) {
        if self.vec_worker.is_empty() {
            return;
        }
        let mut idx = 0;
        loop {
            match self.loop_recv(&self.vec_worker[idx]) {
                Ok(()) => {
                    idx += 1;
                }
                Err(err) => {
                    error!("{}", err);
                    self.vec_worker.remove(idx);
                }
            }
            if idx >= self.vec_worker.len() {
                break;
            }
        }
    }

    fn loop_recv(&self, worker: &Worker<SqlTaskEnum, ()>) -> Result<(), String> {
        let mut idx = 0;
        loop {
            match worker.receiver() {
                RecvResEnum::Empty => {
                    return Ok(());
                }
                RecvResEnum::Data(SqlTaskEnum::QueryTask(mut sql_task)) => {
                    (sql_task.callback)(sql_task.result);
                }
                RecvResEnum::Data(SqlTaskEnum::AlterTask(mut sql_task)) => {
                    (sql_task.callback)(sql_task.result);
                }
                RecvResEnum::Disconnected => {
                    return Err(format!("Worker:{} Disconnected", worker.get_name()));
                }
            }
            idx += 1;
            if idx >= self.single_max_task_num {
                return Ok(());
            }
        }
    }
}
