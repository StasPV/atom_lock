use std::{cell::UnsafeCell, collections::VecDeque, mem::MaybeUninit, ops::{Deref, DerefMut}, sync::{atomic::{fence, AtomicBool, AtomicU64, AtomicUsize, Ordering}, 
            Condvar, 
            Mutex
        }, thread, time::{Duration, Instant}
};

use rand::Rng;
const COUNT: i32 = 20;


#[allow(dead_code)]
fn thread_park() {
    let queue = Mutex::new(VecDeque::new());
    thread::scope(|s| {
        let t = s.spawn(|| loop {
            let item = queue.lock().unwrap().pop_front();
            if let Some(item) = item {
                dbg!(item);
                if item == COUNT - 1 {
                    break;
                }
            } else {
                thread::park();
            }
        });

        for i in 0..COUNT {
            queue.lock().unwrap().push_back(i);
            t.thread().unpark();
            thread::sleep(Duration::from_millis(100));
        }
    });
}

#[allow(dead_code)]
fn thread_condvar(){
    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();
    thread::scope(|s|{
        s.spawn(||{
            let border = COUNT -1;
            loop {
                let mut q = queue.lock().unwrap();
                let item:i32 = loop {
                    if let Some(item) = q.pop_front(){
                        break item;
                    }
                    else {
                        q = not_empty.wait(q).unwrap();
                    }
                };
                drop(q);
                dbg!(item);
                if item == border {
                    break;
                }
            }
        });

        for i in 0..COUNT{
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_millis(100));
        }
    });
    println!("Завершение! Поток остановлен.");
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn atomic_stop(){
    static STOP:AtomicBool = AtomicBool::new(false);

    let background_thread = thread::spawn(||{
        while !STOP.load(Ordering::Relaxed) {
            let i = 10;
            thread::sleep(Duration::from_millis(100));            
        }
    });

    for line in std::io::stdin().lines(){
        match line.unwrap().as_str() {
            "help"=> println!("comands: help, stop"),
            "stop"=> {
                // background_thread.thread().unpark();
                break;
            },
            cmd => println!("unknown command: {cmd:?}"),
        }
    }
    STOP.store(true, Ordering::Relaxed);
    background_thread.join().unwrap();
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn process_thread(){
    let main_thread = &thread::current();
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);
    thread::scope(|s|{
        for t in 0..4{
            s.spawn(move||{
                for i in 1..=100{
                    let start = Instant::now();
                    let x = get_x(t);
                    let time_taken = start.elapsed().as_micros() as u64;
                    
                    num_done.fetch_add(1, Ordering::Relaxed);
                    total_time.fetch_add(time_taken, Ordering::Relaxed);
                    max_time.fetch_max(time_taken, Ordering::Relaxed);
                    main_thread.unpark();
                }
            });
        }

        loop{
            let total_time = Duration::from_micros(total_time.load(Ordering::Relaxed));
            let max_time = Duration::from_micros(max_time.load(Ordering::Relaxed));
            let n = num_done.load(Ordering::Relaxed);
            match n {
                0 => println!("Working.. nothing done yet."),
                100 => break,
                _ => println!("Working.. {n}/100 done, {:?} average, {:?} peak", total_time/n as u32, max_time)
            }
            thread::park_timeout(Duration::from_secs(1));
        }
    });
    let duration = Duration::from_micros(total_time.load(Ordering::Relaxed));
    println!("Финиш! Общее время выполнения: {}", duration.as_secs());
}

fn get_x(start:u64)-> u64{
    static X:AtomicU64 = AtomicU64::new(0);
    let mut x = X.load(Ordering::Relaxed);
    if x == 0 {
        let mut rng = rand::thread_rng(); 
        let a = rng.gen_range(start..100);
        let b = (a +4) * 100;
        x = match X.compare_exchange(0, b, Ordering::Relaxed, Ordering::Relaxed){
            Ok(_) => b,
            Err(k) => k,
        };
    }
    thread::sleep(Duration::from_millis(200));
    x
}

#[allow(dead_code)]
fn fence_thread(){
    static mut DATA:[u64; 10] = [0; 10];
    const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
    static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

    for i in 0..10 {
        thread::spawn(move||{
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(200..700))); // Установим задержку выполнения потока для наглядности примера.
            let data: u64 = rng.gen_range(0..100); // просто генерируем тестовые данные
            unsafe{ DATA[i] = data};
            READY[i].store(true, Ordering::Release);
        });
    }

    thread::sleep(Duration::from_millis(500));
    let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Ordering::Relaxed));
    if ready.contains(&true){
        fence(Ordering::Acquire);
        for i in 1..10{
            if ready[i]{
                println!("data{i} = {}", unsafe{DATA[i]});
            }
        }
    }
    println!("Финиш!");
}

#[allow(dead_code)]
struct SpinLock<T>{
    locked:AtomicBool,
    value: UnsafeCell<T>,
}
#[allow(dead_code)]
impl<T> SpinLock<T>{
    const fn new(value: T)->Self{
        Self{
            locked:AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    fn lock(&self)-> Guard<T> {
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

#[allow(dead_code)]
fn spinlock_guard(){
    todo!("код из примера пока не работает...")
    // 
    // let spin = SpinLock::new(Vec::new());
    // thread::scope(|s|{
    //     s.spawn(|| {
    //         let mut a = spin.lock();
    //         a.push(100 as u64);
    //             });
    //     s.spawn(||{
    //         let mut g = spin.lock();
    //         g.push(200);
    //         g.push(200);
    //     });
    // });
    // let g = spin.lock();
    // assert!(g.as_slice() == [1,2,2] || g.as_slice() == [2,2,1])
}

#[allow(dead_code)]
struct SimpleChanel<T>{
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

#[allow(dead_code)]
impl<T> SimpleChanel<T>{
    fn new()->Self{
        Self{
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    fn send(&self, message: T){
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    fn receive(&self) -> T{
        let mut b = self.queue.lock().unwrap();
        loop {
            if let Some(message) = b.pop_front(){
                return message;
            }
            b = self.item_ready.wait(b).unwrap();
        }
    }
}

#[allow(dead_code)]
pub fn simple_chanel(){
    let chanel: SimpleChanel<u64> = SimpleChanel::new();
    thread::scope(|s|{
        s.spawn(||{
            chanel.send(100);
        });
        s.spawn(||{
            let message = chanel.receive();
            println!("Получено сообщение: {}", message);
        });
    });
}

struct MonoChanel<T>{
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

    unsafe fn send(&self, message: T){
        (*self.message.get()).write(message);
        self.ready.store(true, Ordering::Release);
    }

    fn is_ready(&self)-> bool{
        self.ready.load(Ordering::Acquire)
    }

    unsafe fn receive(&self)-> T{
        (*self.message.get()).assume_init_read()
    }
}