use std::{cell::UnsafeCell, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

#[allow(dead_code)]
pub struct SpinLock<T>{
    locked:AtomicBool,
    value: UnsafeCell<T>,
}
#[allow(dead_code)]
impl<T> SpinLock<T>{
    pub const fn new(value: T)->Self{
        Self{
            locked:AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self)-> Guard<T> {
        while self.locked.swap(true, Ordering::Acquire){
            std::hint::spin_loop();
        }
        Guard{ lock: self}
    }
}

#[allow(dead_code)]
struct Guard<'a, T>{
    lock: &'a SpinLock<T>,
}

#[allow(dead_code)]
impl<T> Deref for Guard<'_, T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe{&*self.lock.value.get()}
    }
}

#[allow(dead_code)]
impl<T> DerefMut for Guard<'_, T>{
    fn deref_mut(&mut self) -> &mut T {
        unsafe{&mut *self.lock.value.get()}
    }
}

impl<T> Drop for Guard<'_, T>{
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}
