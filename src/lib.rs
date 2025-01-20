
use rand::Rng;
use std::{
    collections::VecDeque, fmt::Binary, ops::Deref, sync::{
        atomic::{fence, AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Condvar, Mutex,
    }, thread::{self}, time::{Duration, Instant}
};

const COUNT: i32 = 20;

#[allow(dead_code)]
pub fn thread_park() {
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
pub fn thread_condvar() {
    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();
    thread::scope(|s| {
        s.spawn(|| {
            let border = COUNT - 1;
            loop {
                let mut q = queue.lock().unwrap();
                let item: i32 = loop {
                    if let Some(item) = q.pop_front() {
                        break item;
                    } else {
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

        for i in 0..COUNT {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_millis(100));
        }
    });
    println!("Завершение! Поток остановлен.");
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn atomic_stop() {
    static STOP: AtomicBool = AtomicBool::new(false);

    let background_thread = thread::spawn(|| {
        while !STOP.load(Ordering::Relaxed) {
            let i = 10;
            thread::sleep(Duration::from_millis(100));
        }
    });

    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("comands: help, stop"),
            "stop" => {
                // background_thread.thread().unpark();
                break;
            }
            cmd => println!("unknown command: {cmd:?}"),
        }
    }
    STOP.store(true, Ordering::Relaxed);
    background_thread.join().unwrap();
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn process_thread() {
    let main_thread = &thread::current();
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);
    thread::scope(|s| {
        for t in 0..4 {
            s.spawn(move || {
                for i in 1..=100 {
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

        loop {
            let total_time = Duration::from_micros(total_time.load(Ordering::Relaxed));
            let max_time = Duration::from_micros(max_time.load(Ordering::Relaxed));
            let n = num_done.load(Ordering::Relaxed);
            match n {
                0 => println!("Working.. nothing done yet."),
                100 => break,
                _ => println!(
                    "Working.. {n}/100 done, {:?} average, {:?} peak",
                    total_time / n as u32,
                    max_time
                ),
            }
            thread::park_timeout(Duration::from_secs(1));
        }
    });
    let duration = Duration::from_micros(total_time.load(Ordering::Relaxed));
    println!("Финиш! Общее время выполнения: {}", duration.as_secs());
}

fn get_x(start: u64) -> u64 {
    static X: AtomicU64 = AtomicU64::new(0);
    let mut x = X.load(Ordering::Relaxed);
    if x == 0 {
        let mut rng = rand::thread_rng();
        let a = rng.gen_range(start..100);
        let b = (a + 4) * 100;
        x = match X.compare_exchange(0, b, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => b,
            Err(k) => k,
        };
    }
    thread::sleep(Duration::from_millis(200));
    x
}

#[allow(dead_code)]
pub fn fence_thread() {
    static mut DATA: [u64; 10] = [0; 10];
    const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
    static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

    for i in 0..10 {
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(200..700))); // Установим задержку выполнения потока для наглядности примера.
            let data: u64 = rng.gen_range(0..100); // просто генерируем тестовые данные
            unsafe { DATA[i] = data };
            READY[i].store(true, Ordering::Release);
        });
    }

    thread::sleep(Duration::from_millis(500));
    let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Ordering::Relaxed));
    if ready.contains(&true) {
        fence(Ordering::Acquire);
        for i in 1..10 {
            if ready[i] {
                println!("data{i} = {}", unsafe { DATA[i] });
            }
        }
    }
    println!("Финиш!");
}

mod spin_lock;
use spin_lock::SpinLock;
#[allow(dead_code)]
pub fn spinlock_guard() {
    let spin = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| {
                    let mut a = spin.lock();
                    a.push(1);
        });
        s.spawn(|| {
                    let mut g = spin.lock();
                    g.push(2);
                    g.push(3);
        });
    });
    let g = spin.lock();
    let slice = g.as_slice();
    assert!(slice == [1, 2, 3] || slice == [2, 3, 1]);
    println!("Работа потоков завершена. Состояние: {:?}", slice);
}

mod simple_channel;
use simple_channel::SimpleChanel;
#[allow(dead_code)]
pub fn simple_chanel() {
    let chanel: SimpleChanel<u64> = SimpleChanel::new();
    thread::scope(|s| {
        s.spawn(|| {
            chanel.send(100);
        });
        s.spawn(|| {
            let message = chanel.receive();
            println!("Получено сообщение: {}", message);
        });
    });
}

mod mono_channel;
use mono_channel::MonoChanel;
#[allow(dead_code)]
pub fn mono_chanel() {
    let chanel: MonoChanel<u64> = MonoChanel::new();
    let thd = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            chanel.send(100);
            thd.unpark();
        });
        s.spawn(|| loop {
            if chanel.is_ready() == true {
                let message: u64;
                message = chanel.receive();
                println!("Получено сообщение: {}", message);
                break;
            }
            thread::park();
        });
    });
}

mod channel;
use channel::Channel;
#[allow(dead_code)]
pub  fn channel(){
    let mut chanel = Channel::new();
    thread::scope(|s|{
        let (sender, receiver) = chanel.split();
        s.spawn(move||{
            sender.send("hello world");
        });
        println!("Получено сообщение: {}", receiver.receive());
    })
}

mod arc;
use arc::Arc;
pub fn arc(){
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);
    struct DetectDrop;
    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Ordering::Relaxed);
        }
    }

    let x = Arc::new(("hello", DetectDrop));
    let y = Arc::downgrade(&x);
    let z = Arc::downgrade(&x);

    let t = thread::spawn(move||{
        let y = y.upgrade().unwrap();
        assert_eq!(y.0, "hello");
    });
    assert_eq!(x.0, "hello");

    t.join().unwrap();
    assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);
    assert!(z.upgrade().is_some());
    drop(x);
    assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
    assert!(z.upgrade().is_none());
    println!("Тестирование Arc завершено успешно!");
}

pub fn binary_math(){
    let num_a:u8 = 0b0101;
    let num_b:u8 = 0b0110;
    println!("Результат - {:08b}", (num_a&num_b));
    println!("число {num_a} в двоичном формате: {num_a:04b}");
    println!("число {num_b:04b} в десятичном формате: {num_b}");
}
