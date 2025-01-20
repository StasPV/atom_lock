use std::{hint::black_box, ops::Deref, sync::atomic::{AtomicU64, Ordering}, thread, time::Instant};

#[allow(dead_code)]
#[repr(align(64))] // выравнивание границы структуры.
struct Aligned(AtomicU64);
impl Deref for Aligned{
    type Target = AtomicU64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
static A:[Aligned; 3] =[
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
    Aligned(AtomicU64::new(0)),
];

#[allow(dead_code)]
static B:[AtomicU64; 3] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

pub fn test_cashe_delay_from_aligned(){
    black_box(&A);
    thread::spawn(||{
        loop{
            A[0].store(1, Ordering::Relaxed);
            A[2].store(1, Ordering::Relaxed);
        }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000{
        black_box(A[1].load(Ordering::Relaxed));
    }
    println!("Скорость с выравниванием: {:?}", start.elapsed().as_millis());
}

pub fn test_cashe_delay(){
    black_box(&B);
    thread::spawn(||{
        loop{
            B[0].store(1, Ordering::Relaxed);
            B[2].store(1, Ordering::Relaxed);
        }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000{
        black_box(B[1].load(Ordering::Relaxed));
    }
    println!("Скорость без выравнивания: {:?}", start.elapsed().as_millis());
}