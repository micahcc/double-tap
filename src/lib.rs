use std::time::Duration;
use std::time::Instant;

const SIGSET_SIZE: usize = std::mem::size_of::<libc::sigset_t>();

pub fn setup_graceful_shutdown() {
    // Create an empty block of memory that will become our sigset
    let mut sigset =
        unsafe { std::mem::transmute::<[u8; SIGSET_SIZE], libc::sigset_t>([0_u8; SIGSET_SIZE]) };
    unsafe { libc::sigemptyset(&mut sigset) };
    unsafe { libc::sigaddset(&mut sigset, libc::SIGINT) };
    unsafe { libc::sigaddset(&mut sigset, libc::SIGTERM) };

    // finally setup
    unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, &mut sigset, std::ptr::null_mut()) };
}

pub fn trigger_shutdown() {}

pub fn shutdown_triggered() -> bool {
    return wait_for_shutdown_with_timeout(Duration::ZERO);
}

fn unmask_signals_in_current_thread() {
    let mut sigset =
        unsafe { std::mem::transmute::<[u8; SIGSET_SIZE], libc::sigset_t>([0_u8; SIGSET_SIZE]) };
    unsafe { libc::sigemptyset(&mut sigset) };
    unsafe { libc::sigaddset(&mut sigset, libc::SIGINT) };
    unsafe { libc::sigaddset(&mut sigset, libc::SIGTERM) };

    // signal! Unmask so that a second signal will kill the program
    eprintln!("unmasking");
    unsafe { libc::pthread_sigmask(libc::SIG_UNBLOCK, &mut sigset, std::ptr::null_mut()) };
}

pub fn real_wait_for_signal(timeout: Duration) -> bool {
    let mut sigset =
        unsafe { std::mem::transmute::<[u8; SIGSET_SIZE], libc::sigset_t>([0_u8; SIGSET_SIZE]) };
    unsafe { libc::sigemptyset(&mut sigset) };
    unsafe { libc::sigaddset(&mut sigset, libc::SIGINT) };
    unsafe { libc::sigaddset(&mut sigset, libc::SIGTERM) };

    // cap the actual wait time at 1 second, this is only for the real wait
    let tv_nsec = timeout.as_nanos().min(1_000_000_000) as i64;
    let wait_time = libc::timespec { tv_nsec, tv_sec: 0 };

    let ret = unsafe { libc::sigtimedwait(&sigset, std::ptr::null_mut(), &wait_time) };
    if ret == -1 {
        // no signal
        eprintln!("no signal");
        return false;
    }
    return true;
}

pub fn wait_for_shutdown_with_timeout(timeout: Duration) -> bool {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::sync::Condvar;
    use std::sync::Mutex;

    if TRIGGERED.load(Ordering::Relaxed) {
        unmask_signals_in_current_thread();
        return true;
    }

    // Only 1 waiter of sigtimedwait will receive the event, so we'll force a single
    // thread into the waiting below, everyone else will wait for THAT waiter
    static REAL_WAITER_CV: Condvar = Condvar::new();
    static REAL_WAITER_MTX: Mutex<bool> = Mutex::new(false);
    static TRIGGERED: AtomicBool = AtomicBool::new(false);

    let now = Instant::now();
    let end_time = now + timeout;
    {
        // wait until no one is waiting, while we are waiting:
        // if we time out we'll just return false
        // if we see a trigger we'll return true
        {
            let mut waiter_active = REAL_WAITER_MTX.lock().expect("Locking");
            loop {
                if TRIGGERED.load(Ordering::Relaxed) {
                    unmask_signals_in_current_thread();
                    return true;
                }

                if !*waiter_active {
                    // we'll be the waiter now
                    *waiter_active = true;
                    break;
                }

                let remaining = end_time - Instant::now();
                if remaining <= Duration::ZERO {
                    // just checked and there haven't been triggers and we've run out of time
                    return false;
                }

                let result = REAL_WAITER_CV
                    .wait_timeout(waiter_active, remaining)
                    .expect("lock");
                waiter_active = result.0;
                if result.1.timed_out() {
                    eprintln!("timed out");
                } else {
                    eprintln!("notified");
                }
            }
        }

        // to break out we set a waiter as active, thats us
        let remaining = end_time - Instant::now();
        if real_wait_for_signal(remaining) {
            // triggered, increment
            eprintln!("set triggered");
            let _ = TRIGGERED.fetch_or(true, Ordering::Relaxed);
            unmask_signals_in_current_thread();
            return true;
        }

        // we're done, let someone else try
        *REAL_WAITER_MTX.lock().expect("Locking") = false;
        eprintln!("notify");
        REAL_WAITER_CV.notify_all();
    }

    return false;
}

pub fn wait_for_shutdown() {
    while !wait_for_shutdown_with_timeout(Duration::from_secs(1)) {}
}
