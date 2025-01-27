use std::marker::PhantomData;
use std::thread::{self, Thread};
use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};
use std::sync::atomic::Ordering;

pub struct Channel<T>{
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

pub struct Sender<'a, T>{
    channel: &'a Channel<T>,
    receiving_thread: Thread,
}

pub struct Receiver<'a, T>{
    channel: &'a Channel<T>,
    _no_send: PhantomData<*const()>
}

impl<T> Channel<T>{
    pub const fn new()-> Self{
        Self{
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    pub fn split<'a>(&'a mut self)-> (Sender<'a, T>, Receiver<'a, T>){
        *self = Self::new();
        (Sender{
            channel: self, 
            receiving_thread: thread::current()
        }, 
            Receiver{
                channel: self,
                _no_send: PhantomData
            })
    }
}

impl<T> Drop for Channel<T>{
    fn drop(&mut self) {
        if *self.ready.get_mut(){
            unsafe {self.message.get_mut().assume_init_drop();}
        }
    }
}

impl<T> Sender<'_, T>{
    pub fn send(self, message:T){
        unsafe{(*self.channel.message.get()).write(message)};
        self.channel.ready.store(true, Ordering::Release);
        self.receiving_thread.unpark();
    }
}

impl<T> Receiver<'_, T>{
    #[allow(dead_code)]
    pub fn is_ready(&self)-> bool{
        self.channel.ready.load(Ordering::Relaxed)
    }

    pub fn receive(self)-> T{
        while !self.channel.ready.swap(false, Ordering::Acquire){
            thread::park();
        }
        unsafe{(*self.channel.message.get()).assume_init_read()}
    }
}