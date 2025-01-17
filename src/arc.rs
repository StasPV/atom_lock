use std::{cell::UnsafeCell, mem::ManuallyDrop, ops::Deref, ptr::NonNull, sync::atomic::{fence, AtomicUsize, Ordering}, usize};

struct ArcData<T>{
    data_ref_count: AtomicUsize,
    alloc_ref_count: AtomicUsize,
    data: UnsafeCell<ManuallyDrop<T>>,
}

pub struct Arc<T>{
    ptr: NonNull<ArcData<T>>,
}
unsafe impl<T: Send + Sync> Sync for Arc<T>{}
unsafe impl<T: Send + Sync> Send for Arc<T>{}

pub struct Weak<T>{
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Sync for Weak<T>{}
unsafe impl<T: Send + Sync> Send for Weak<T>{}

impl<T> Arc<T>{
    pub fn new(data: T)->Arc<T>{
        Arc{
            ptr: NonNull::from(Box::leak(Box::new(ArcData{
                data_ref_count: AtomicUsize::new(1),
                alloc_ref_count: AtomicUsize::new(1),
                data: UnsafeCell::new(ManuallyDrop::new(data)),
            }))),
        }
    }

    #[allow(dead_code)]
    pub fn get_mut(arc: &mut Self)-> Option<&mut T>{
        if arc.data().alloc_ref_count.compare_exchange(1, usize::MAX, Ordering::Acquire, Ordering::Relaxed).is_err(){
            return None;
        }
        let is_unique = arc.data().data_ref_count.load(Ordering::Relaxed) == 1;
        arc.data().alloc_ref_count.store(1, Ordering::Release);
        if !is_unique{
            return None;
        }
        fence(Ordering::Acquire);
        unsafe{Some(&mut *arc.data().data.get())}
    }

    pub fn downgrade(arc: &Self)->Weak<T>{
        let mut n = arc.data().alloc_ref_count.load(Ordering::Relaxed);
        loop {
            if n == usize::MAX{
                std::hint::spin_loop();
                n = arc.data().alloc_ref_count.load(Ordering::Relaxed);
                continue;
            }
            assert!(n < usize::MAX - 1);

            if let Err(e) = arc.data().alloc_ref_count.compare_exchange_weak(n, n+1, Ordering::Acquire, Ordering::Relaxed){
                n = e;
                continue;
            }
            return Weak{ptr: arc.ptr};
        }
    }

    fn data(&self)->&ArcData<T>{
        unsafe{self.ptr.as_ref()}
    }
}

impl<T> Weak<T>{
    fn data(&self)->&ArcData<T>{
        unsafe{self.ptr.as_ref()}
    }

    pub fn upgrade(&self)->Option<Arc<T>>{
        let mut n = self.data().data_ref_count.load(Ordering::Relaxed);
        loop{
            if n == 0{
                return None;
            }
            assert!(n < usize::MAX);
            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(n, n+1, Ordering::Relaxed, Ordering::Relaxed){
                n = e;
                continue;
            }
            return Some(Arc{ptr:self.ptr});
        }
    }
}

impl<T> Deref for Arc<T>{
    type Target = T;
    fn deref(&self) -> &T {
        unsafe{&*self.data().data.get()}
    }
}

impl<T> Clone for Weak<T>{
    fn clone(&self) -> Self {
        if(self.data().alloc_ref_count.fetch_add(1, Ordering::Relaxed)) > usize::MAX/2{
            std::process::abort();
        }
        Weak { ptr: self.ptr }
    }
}
impl<T> Clone for Arc<T>{
    fn clone(&self) -> Self {
        if self.data().data_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX/2{
            std::process::abort();
        }
        Arc { ptr: self.ptr }
    }
}

impl <T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Ordering::Release) == 1{
            fence(Ordering::Acquire);
            unsafe{drop(Box::from_raw(self.ptr.as_ptr()));}
        }
    }
}
impl<T> Drop for Arc<T>{
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Ordering::Release) == 1{
            fence(Ordering::Acquire);
            unsafe {
                ManuallyDrop::drop(&mut *self.data().data.get());
            }
            drop(Weak{ptr:self.ptr});
        }
    }
}