use std::env;

use atom_lock as atl;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let run_number = env::var("RUN_NUMBER").unwrap().parse::<u32>().unwrap();
    match run_number {
        0 => atl::thread_park(),
        1 => atl::thread_condvar(),
        2 => atl::atomic_stop(),
        3 => atl::process_thread(),
        4 => atl::fence_thread(),
        5 => atl::spinlock_guard(),
        6 => atl::simple_chanel(),
        7 => atl::mono_chanel(),
        8 => atl::channel(),
        9 => atl::arc(),
        _ => println!("неизвестный параметр запуска"), 
    }
}
