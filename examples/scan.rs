extern crate pmw3901;
use std::thread;
use std::time;

fn main() {
    let mut pmw3901 = pmw3901::Pmw3901::new(0, 0).unwrap();
    pmw3901.init().unwrap();

    loop {
        let sample = pmw3901.read_sample().unwrap();
        println!("x: {}, y: {}", sample.x, sample.y);
        thread::sleep(time::Duration::from_millis(100));
    }
}
