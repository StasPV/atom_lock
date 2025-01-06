use std::{collections::VecDeque, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Condvar, Mutex}, thread, time::Duration};
const COUNT: i32 = 20;
fn main() {
    process_thread();
}

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
    let num_done = AtomicUsize::new(0);
    let main_thread = thread::current();
    thread::scope(|s|{
        s.spawn(||{
            for i in 0..100{
                let a = i*2;
                let b = a +4;
                let c = b * 100;
                thread::sleep(Duration::from_millis(700));
                num_done.store(i+1, Ordering::Relaxed);
                main_thread.unpark();
            }
        });

        loop{
            let n = num_done.load(Ordering::Relaxed);
            if n == 100 {break;}
            println!("Working.. {n}/100 done");
            thread::park_timeout(Duration::from_secs(1));
        }
    });
    println!("Финиш!");
}