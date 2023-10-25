use std::marker::Send;
use std::panic::UnwindSafe;
use std::sync::Arc;
use std::thread;
use std::thread::Builder;

mod thread_data;

pub struct ThreadPool<T> {
    thread_data: Arc<thread_data::ThreadData<T>>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl<T: Send + UnwindSafe + 'static> ThreadPool<T> {
    pub fn new(num_threads: u64) -> ThreadPool<T> {
        let mut thread_pool: ThreadPool<T> = ThreadPool::<T> {
            thread_data: Arc::new(thread_data::ThreadData::<T>::new()),
            threads: vec![],
        };

        let mut count: u64 = 0;
        loop {
            let thread_data_copy: Arc<thread_data::ThreadData<T>> =
                Arc::clone(&thread_pool.thread_data);

            let thread_builder: Builder = thread::Builder::new();
            let handle: thread::JoinHandle<_> =
                match thread_builder.spawn(move || thread_data_copy.thread_work()) {
                    Ok(v) => v,
                    Err(_) => break,
                };

            thread_pool.threads.push(handle);

            if num_threads != 0 {
                count = count + 1;
            }

            if count == num_threads && num_threads != 0 {
                break;
            }
        }

        return thread_pool;
    }

    pub fn add_job(&self, job: fn(data: T) -> Result<(), ()>, data: T) -> Result<(), ()> {
        return match self.thread_data.is_end_work() {
            false => self.thread_data.add_job(job, data),
            true => Err(()),
        };
    }

    pub fn jobs_finished(&self) -> Result<bool, ()> {
        if self.thread_data.is_error() {
            return Err(());
        }

        let mut all_threads_finished: bool = true;
        for t in &self.threads {
            if t.is_finished() && !self.thread_data.is_end_work() {
                return Err(());
            } else if !t.is_finished() {
                all_threads_finished = false;
            }
        }

        if self.thread_data.is_end_work() {
            return match all_threads_finished {
                true => Ok(true),
                false => Ok(false),
            };
        }

        if !self.thread_data.is_jobs_empty() || self.thread_data.get_working_thread_count() != 0 {
            return Ok(false);
        }

        return Ok(true);
    }

    pub fn set_end_work(&self) {
        self.thread_data.set_end_work();
    }
}
