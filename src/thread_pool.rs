// use std::{
//     collections::VecDeque,
//     sync::{Arc, Mutex},
//     thread,
//     time::Duration,
// };

// type RequestQueue = Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>;

// pub struct ThreadPool {
//     queue: RequestQueue,
// }

// impl ThreadPool {
//     pub fn new(max_threads: u32) -> ThreadPool {
//         let thread_pool = ThreadPool {
//             queue: Arc::new(Mutex::new(VecDeque::new())),
//         };

//         // spawn threads
//         (0..max_threads).for_each(|_| {
//             let queue = Arc::clone(&thread_pool.queue);
//             thread::spawn(move || {
//                 loop {
//                     // attempt to dequeue a request
//                     let mut request = None;
//                     {
//                         let mut lock = queue.try_lock();
//                         if let Ok(ref mut queue) = lock {
//                             request = queue.pop_front();
//                         }
//                     }

//                     if let Some(request) = request {
//                         request();
//                     } else {
//                         thread::sleep(Duration::from_millis(100));
//                     }
//                 }
//             });
//         });
//         thread_pool
//     }

//     pub fn enqueue(&self, task: impl FnOnce() + Send + 'static) {
//         self.queue.lock().unwrap().push_back(Box::new(task));
//     }
// }

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    queue: Arc<Mutex<VecDeque<Job>>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&queue)));
        }

        ThreadPool { workers, queue }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.lock().unwrap().push_back(Box::new(f));
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

// --snip--

impl Worker {
    fn new(id: usize, queue: Arc<Mutex<VecDeque<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let mut job = None;

            {
                let mut lock = queue.try_lock();
                if let Ok(ref mut queue) = lock {
                    job = queue.pop_front();
                }
            }

            if let Some(job) = job {
                println!("Worker {id} got a job; executing.");
                job();
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        });

        Worker { id, thread }
    }
}
