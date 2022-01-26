use std::sync::{Mutex, MutexGuard, Condvar};

pub struct Wait {
    pub lock: Mutex<bool>,
    pub cv: Condvar,
}

impl Wait {
    pub fn new() -> Wait {
        Wait {
            lock: Mutex::new(false),
            cv: Condvar::new(),
        }
    }

    pub fn update(&self) -> WaitGuard {
        let pending = self.lock.lock().unwrap();
        assert!(!*pending);
        WaitGuard {
            lock: pending,
            cv: &self.cv,
        }
    }

    pub fn wait(&self) {
        let _ = self.cv.wait_while(self.lock.lock().unwrap(), |ready| *ready).unwrap();
    }
}

pub struct WaitGuard<'a> {
    pub lock: MutexGuard<'a, bool>,
    pub cv: &'a Condvar,
}

impl Drop for WaitGuard<'_> {
   fn drop(&mut self) {
       *self.lock = true;
       self.cv.notify_all();
   }
}
