use std::time::Duration;

use double_tap::setup_graceful_shutdown;
use double_tap::shutdown_triggered;

fn worker() {
    while !shutdown_triggered() {
        // do stuff
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    setup_graceful_shutdown();

    std::thread::spawn(worker);

    while !shutdown_triggered() {
        // do stuff
        std::thread::sleep(Duration::from_millis(1));
    }
    eprintln!("Being bad and taking overly long to die");
    std::thread::sleep(Duration::from_secs(1000));

    eprintln!("Shutting down happily");
}
