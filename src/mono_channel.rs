use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::{AtomicBool, Ordering}};
pub struct MonoChanel<T>{
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
    in_use: AtomicBool,
}

unsafe impl<T> Sync for MonoChanel<T> where T: Send{}

impl<T> MonoChanel<T>{
    pub const fn new()->Self{
        Self { 
            message: UnsafeCell::new(MaybeUninit::uninit()), 
            ready: AtomicBool::new(false),
            in_use: AtomicBool::new(false),
         }
    }

    pub fn send(&self, message: T){
        if self.in_use.swap(true, Ordering::Relaxed){
            panic!("can't send more than one message");
        }
        unsafe{(*self.message.get()).write(message)};
        self.ready.store(true, Ordering::Release);
    }

    pub fn is_ready(&self)-> bool{
        self.ready.load(Ordering::Relaxed)
    }

    pub fn receive(&self)-> T{
        if !self.ready.swap(false, Ordering::Acquire){
            panic!("no message available");
        }
        unsafe{(*self.message.get()).assume_init_read()}
    }
}

impl<T> Drop for MonoChanel<T>{
    fn drop(&mut self) {
        if *self.ready.get_mut(){
            unsafe {self.message.get_mut().assume_init_drop()}
        }
    }
}