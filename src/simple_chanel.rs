use std::{collections::VecDeque, sync::{Condvar, Mutex}};

#[allow(dead_code)]
pub struct SimpleChanel<T>{
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

#[allow(dead_code)]
impl<T> SimpleChanel<T>{
    pub fn new()->Self{
        Self{
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T){
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    pub fn receive(&self) -> T{
        let mut b = self.queue.lock().unwrap();
        loop {
            if let Some(message) = b.pop_front(){
                return message;
            }
            b = self.item_ready.wait(b).unwrap();
        }
    }
}