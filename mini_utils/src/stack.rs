#![allow(dead_code)]

struct Stack<T>{
    pub vec: Vec<T>
}

impl<T> Stack<T> {
    pub fn new(capacity: u16)->Self{
        Stack{vec:Vec::with_capacity(capacity as usize)}
    }

    pub fn pop(&mut self)->Option<T>{
        self.vec.pop()
    }

    pub fn push(&mut self, data: T)->bool{
        if self.vec.len() == self.vec.capacity(){
            return false;
        }else{
            self.vec.push(data);
            return true;
       }
    }
}

#[test]
fn test(){
    let mut stack = Stack::new(15);

    for i in 0..18{
        if !stack.push(i){
            println!("push:{} error", i);
            println!("len:{}, cap:{}", stack.vec.len(), stack.vec.capacity()); 
            break;
        }
    }

    println!("len:{}, cap:{}", stack.vec.len(), stack.vec.capacity()); 

    for i in 0.. 19{
        if None == stack.pop(){
            println!("push:{} error", i);
            println!("len:{}, cap:{}", stack.vec.len(), stack.vec.capacity()); 
            break;
        }
    }
}