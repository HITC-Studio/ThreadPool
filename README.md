# Create a Thread Pool

Create a Thread Pool using the turbo:fish of Type T, in which Type T is the type of variable the passed in function will take.

## Usage in other Libraries or Binaries

### Cargo.toml
```
...
[dependencies]
...
thread_pool = { path = "<Path/to/thread_pool>" }
...
```

### Source Code
```
...
use thread_pool::*;
...

fn my_fn(data: Arc<T>) -> Result<(), ()> {
    // Do something with data
    return Ok(()); // or return Err(());
}

fn some_fn() {
  ...
  let data: Arc<T> = Arc::new(T {...});
	let thread_pool: ThreadPool<Arc<T>> = ThreadPool::new(50);
  for _ in 0..num_jobs {
    let data_copy: Arc<T> = Arc::clone(&data);
    thread_pool.add_job(my_fn, data_copy);
  }
  ...
  // Do other things
  // Signal work should stop or wait till the job queue is empty
  // thread_pool.set_end_work();
  // Wait for work to end
  loop {
    if thread_pool.jobs_finished().unwrap() {
      break;
    }
  }
}
```

## Build
`cargo build` or `cargo build --release`

## Tests
`cargo test`

### Integration Tests
Found at "thread_pool/tests/integration_tests.rs". Tests the public methods of the library, that would be used by other libraries or binaries.
