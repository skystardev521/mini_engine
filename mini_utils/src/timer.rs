use std::collections::VecDeque;
pub struct Timer {
    /// 运行次数
    /// <0无限次数
    //call_num: i16,
    /// 下次运行时间
    expire: u64,
    /// 运行间隔时长
    interval: u32,
    /// 运行次数-1无限
    //call_fn: Box<dyn FnMut(/*&mut Timer*/)>,
    call_fn: Box<dyn TEntiy>,
}

pub trait TEntiy{

    fn call(&mut self);

}

impl Timer {
    
    /*
    /// 
    pub fn new(dely: u32, interval: u32, call_fn: Box<dyn FnMut(/*&mut Timer*/)>) -> Timer
    {
        Timer {
            //call_num,
            interval,
            call_fn,
            expire:0
        }
    }
    */

    pub fn new(dely: u32, interval: u32, call_fn: Box<dyn TEntiy>) -> Timer
    {
        Timer {
            //call_num,
            interval,
            call_fn,
            expire:0
        }
    }

    /*
    //pub(crate) fn call(&mut self){
    pub fn call(&mut self){
        (self.call_fn)();
    }
    */

}

pub struct Timer1<T> {
    pub vec_deque: VecDeque<T>,
}
