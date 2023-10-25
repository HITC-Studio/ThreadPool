use std::panic;
use std::panic::UnwindSafe;
use std::sync::{Condvar, Mutex, MutexGuard};

pub struct ThreadData<T> {
    end_work: Mutex<bool>,
    error: Mutex<bool>,
    jobs: Mutex<Vec<(fn(data: T) -> Result<(), ()>, T)>>,
    jobs_cond: Condvar,
    working_thread_count: Mutex<u64>,
}

impl<T: UnwindSafe> ThreadData<T> {
    pub fn new() -> ThreadData<T> {
        return ThreadData {
            end_work: Mutex::new(false),
            error: Mutex::new(false),
            jobs: Mutex::new(vec![]),
            jobs_cond: Condvar::new(),
            working_thread_count: Mutex::new(0),
        };
    }

    pub fn thread_work(&self) {
        while !self.is_error() && !self.is_end_work() {
            let job_option: Option<(fn(data: T) -> Result<(), ()>, T)> = match self.get_job() {
                Ok(v) => v,
                Err(_) => return,
            };

            match job_option {
                Some(v) => match self.run_job(v.0, v.1) {
                    Err(_) => return,
                    Ok(_) => {}
                },
                None => {}
            }
        }
    }

    fn get_job(&self) -> Result<Option<(fn(data: T) -> Result<(), ()>, T)>, ()> {
        let mut jobs_lock: MutexGuard<'_, Vec<(fn(T) -> Result<(), ()>, T)>> =
            match self.jobs.lock() {
                Err(_) => {
                    self.set_error();
                    return Err(());
                }

                Ok(v) => {
                    if v.is_empty() {
                        match self.jobs_cond.wait(v) {
                            Err(_) => {
                                self.set_error();
                                return Err(());
                            }
                            Ok(v) => v,
                        }
                    } else {
                        v
                    }
                }
            };

        if self.is_error() || self.is_end_work() {
            return Err(());
        }

        if jobs_lock.is_empty() {
            return Ok(None);
        }

        return match jobs_lock.pop() {
            None => {
                self.set_error();
                return Err(());
            }
            Some(v) => {
                self.adjust_working_thread_count(true);
                Ok(Some(v))
            }
        };
    }

    fn run_job(&self, job: fn(data: T) -> Result<(), ()>, data: T) -> Result<(), ()> {
        let job_result: Mutex<Result<(), ()>> = Mutex::new(Ok(()));

        let panic_result = panic::catch_unwind(|| {
            *job_result.lock().unwrap() = job(data);
        });

        self.adjust_working_thread_count(false);

        match panic_result {
            Err(_) => {
                self.set_error();
                return Err(());
            }
            Ok(_) => match *job_result.lock().unwrap() {
                Err(_) => {
                    self.set_error();
                    return Err(());
                }
                Ok(_) => {}
            },
        };

        return Ok(());
    }

    pub fn add_job(&self, job: fn(data: T) -> Result<(), ()>, data: T) -> Result<(), ()> {
        match self.jobs.lock() {
            Ok(mut v) => {
                v.push((job, data));

                self.jobs_cond.notify_one();

                return Ok(());
            }
            Err(_) => {}
        };

        self.set_error();
        return Err(());
    }

    fn set_error(&self) {
        match self.error.lock() {
            Ok(mut v) => *v = true,
            Err(_) => {}
        };

        self.jobs_cond.notify_all();
    }

    pub fn is_error(&self) -> bool {
        return match self.error.lock() {
            Ok(v) => *v,
            Err(_) => true,
        };
    }

    pub fn is_jobs_empty(&self) -> bool {
        match self.jobs.lock() {
            Ok(v) => return v.is_empty(),
            Err(_) => {}
        };

        self.set_error();
        return true;
    }

    pub fn get_working_thread_count(&self) -> u64 {
        match self.working_thread_count.lock() {
            Ok(v) => return *v,
            Err(_) => {}
        };

        self.set_error();
        return 0;
    }

    pub fn set_end_work(&self) {
        match self.end_work.lock() {
            Ok(mut v) => {
                *v = true;
                self.jobs_cond.notify_all();
                return;
            }
            Err(_) => {}
        };

        self.set_error();
    }

    pub fn is_end_work(&self) -> bool {
        return match self.end_work.lock() {
            Ok(v) => *v,
            Err(_) => {
                self.set_error();
                true
            }
        };
    }

    fn adjust_working_thread_count(&self, value: bool) {
        match self.working_thread_count.lock() {
            Err(_) => {}
            Ok(mut v) => match value {
                true => {
                    *v += 1;
                    return;
                }
                false => {
                    *v -= 1;
                    return;
                }
            },
        };

        self.set_error();
    }
}
