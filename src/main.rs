use std::{collections::VecDeque, sync::{Condvar, Mutex}, thread, time::Duration};
const COUNT: i32 = 20;
fn main() {
    condvar();
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

fn condvar(){
    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();
    thread::scope(|s|{
        s.spawn(||{
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
            }
        });

        for i in 0..COUNT{
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_millis(100));
        }
    });
}
