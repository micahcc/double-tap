use std::time::Duration;

use double_tap::setup_graceful_shutdown;
use double_tap::wait_for_shutdown;
use double_tap::wait_for_shutdown_with_timeout;

fn worker() {
    while !wait_for_shutdown_with_timeout(Duration::from_millis(100)) {
        // do stuff
    }
}

fn main() {
    setup_graceful_shutdown();

    std::thread::spawn(worker);
    std::thread::spawn(worker);
    std::thread::spawn(worker);
    std::thread::spawn(worker);
    std::thread::spawn(worker);
    std::thread::spawn(worker);
    std::thread::spawn(worker);
    std::thread::spawn(worker);

    wait_for_shutdown();
    eprintln!("Being bad and taking overly long to die");
    std::thread::sleep(Duration::from_secs(1000));

    eprintln!("Shutting down happily");
}
