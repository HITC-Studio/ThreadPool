use std::sync::{Arc, Mutex};
use thread_pool::ThreadPool;

fn test_fn_simple(data: bool) -> Result<(), ()> {
    return match data {
        true => Ok(()),
        false => Err(()),
    };
}

#[test]
fn create_thread_pool_simple_test() {
    let thread_pool: ThreadPool<bool> = ThreadPool::new(50);

    for _ in 0..500 {
        assert!(thread_pool.add_job(test_fn_simple, true).is_ok());
    }

    loop {
        let result = thread_pool.jobs_finished();
        assert!(result.is_ok());
        if result.unwrap() {
            break;
        }
    }

    thread_pool.set_end_work();
}

#[test]
fn create_thread_pool_simple_end_test() {
    let thread_pool: ThreadPool<bool> = ThreadPool::new(50);

    for _ in 0..500 {
        assert!(thread_pool.add_job(test_fn_simple, true).is_ok());
    }

    thread_pool.set_end_work();

    loop {
        let result = thread_pool.jobs_finished();
        assert!(result.is_ok());
        if result.unwrap() {
            break;
        }
    }
}

struct ComplexData {
    pub string_data: Mutex<Vec<String>>,
}

fn test_fn_complex(data: Arc<ComplexData>) -> Result<(), ()> {
    data.string_data.lock().unwrap().push(String::from("tmp"));
    return Ok(());
}

#[test]
fn create_thread_pool_complex_test() {
    let num_jobs: usize = 500;

    let data: Arc<ComplexData> = Arc::new(ComplexData {
        string_data: Mutex::new(vec![]),
    });

    let thread_pool: ThreadPool<Arc<ComplexData>> = ThreadPool::new(50);

    for _ in 0..num_jobs {
        let data_copy: Arc<ComplexData> = Arc::clone(&data);
        assert!(thread_pool.add_job(test_fn_complex, data_copy).is_ok());
    }

    loop {
        let result = thread_pool.jobs_finished();
        assert!(result.is_ok());
        if result.unwrap() {
            break;
        }
    }

    thread_pool.set_end_work();

    assert_eq!(data.string_data.lock().unwrap().len(), num_jobs);
}
