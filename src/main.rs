use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};
const COUNT:i32 = 20;
fn main() {
    let queue = Mutex::new(VecDeque::new());
    thread::scope(|s|{
        let t = s.spawn(||loop{
            let item = queue.lock().unwrap().pop_front();
            if let Some(item) = item {
                dbg!(item);
                if item == COUNT - 1{
                    break;
                }
            }
            else {
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
