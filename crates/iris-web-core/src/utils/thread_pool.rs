use std::thread::JoinHandle;
use std::sync::{mpsc, Arc, Mutex};

/// A thread pool that can execute closures in parallel.
pub struct ThreadPool {
    workers: Vec<ThreadPoolWorker>,
    sender: mpsc::Sender<ThreadPoolJob>,
}

impl ThreadPool {
    /// Creates a new thread pool.
    pub fn new(size: usize) -> Self {
        if size == 0 {
            // Do not create a thread pool if the size is 0.
            return Self {
                workers: Vec::new(),
                sender: mpsc::channel().0,
            };
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(ThreadPoolWorker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender,
        }
    }

    /// Queues a closure to be executed in the thread pool.
    /// The closure must be `Send` and `'static`.
    pub fn queue<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            #[cfg(debug_assertions)]
            println!("Shutting down worker {}.", worker.id);

            worker.thread.take().unwrap().join().unwrap();
        }
    }
}

/// A worker job in the thread pool.
type ThreadPoolJob = Box<dyn FnOnce() + Send + 'static>;

/// A worker thread in the thread pool.
struct ThreadPoolWorker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPoolWorker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<ThreadPoolJob>>>) -> Self {
        let thread = std::thread::spawn(move || {
            #[cfg(debug_assertions)]
            println!("Worker {id} started.");
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();

                #[cfg(debug_assertions)]
                println!("Worker {id} got a job; executing.");

                job();
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}
