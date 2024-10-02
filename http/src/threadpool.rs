use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A simple thread pool.
///
/// ```
/// use http::threadpool::ThreadPool;
///
/// let pool = ThreadPool::new(3);
/// pool.execute(|| {
///     println!("hello from a thread");
/// });
///
/// pool.execute(|| {
///     println!("hello from a thread");
/// });
/// ```
pub struct ThreadPool {
    pub count: usize,
    pub workers: Vec<Option<thread::JoinHandle<()>>>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// # Panics
    ///
    /// Panics if a thread cannot be spawned.
    #[must_use]
    pub fn new(count: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(count);
        let (tx, rx) = mpsc::channel::<Job>();
        let arx = Arc::new(Mutex::new(rx));
        for _ in 0..count {
            let thread_rx = arx.clone();
            let handle = thread::spawn(move || loop {
                #[expect(clippy::unwrap_used)]
                let message = thread_rx.lock().unwrap().recv();
                if let Ok(job) = message {
                    job();
                } else {
                    println!("shutting down..");
                    break;
                };
            });
            workers.push(Some(handle));
        }

        ThreadPool {
            count,
            workers,
            sender: Some(tx),
        }
    }

    /// # Errors
    ///
    /// Errors if the job cannot  be sent to the mpsc channel.
    ///
    /// # Panics
    ///
    /// Panics if
    pub fn execute<F>(&self, job: F) -> Result<(), mpsc::SendError<Job>>
    where
        F: FnOnce() + Send + 'static,
    {
        #[expect(clippy::expect_used)]
        self.sender
            .as_ref()
            .expect("Sender is None only if the threadpool is dropped")
            .send(Box::new(job))
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        #[expect(clippy::expect_used)]
        drop(
            self.sender
                .take()
                .expect("Sender is only None after threadpool is dropped"),
        );
        for handle in &mut self.workers {
            #[expect(clippy::expect_used)]
            let _ = handle
                .take()
                .expect("Thread handlers are only None after threadpool is dropped")
                .join();
        }
    }
}
