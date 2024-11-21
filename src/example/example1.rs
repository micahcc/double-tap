use double_tap::setup_graceful_shutdown;
use double_tap::shutdown_triggered;

fn main() {
    setup_graceful_shutdown();
    while !shutdown_triggered() {
        // do stuff
    }
}
