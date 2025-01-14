use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::{AtomicBool, Ordering}};

pub struct MonoChanel<T>{
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for MonoChanel<T> where T: Send{}
impl<T> MonoChanel<T>{
    pub const fn new()->Self{
        Self { 
            message: UnsafeCell::new(MaybeUninit::uninit()), 
            ready: AtomicBool::new(false),
         }
    }

    pub unsafe fn send(&self, message: T){
        (*self.message.get()).write(message);
        self.ready.store(true, Ordering::Release);
    }

    pub fn is_ready(&self)-> bool{
        self.ready.load(Ordering::Acquire)
    }

    pub unsafe fn receive(&self)-> T{
        (*self.message.get()).assume_init_read()
    }
}