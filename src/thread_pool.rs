use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

type RequestQueue = Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>;

pub struct ThreadPool {
    queue: RequestQueue,
}

impl ThreadPool {
    pub fn new(max_threads: u32) -> ThreadPool {
        let thread_pool = ThreadPool {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        };

        // spawn threads
        (0..max_threads).for_each(|_| {
            let queue = Arc::clone(&thread_pool.queue);
            thread::spawn(move || {
                loop {
                    // attempt to dequeue a request
                    let mut request = None;
                    {
                        let mut lock = queue.try_lock();
                        if let Ok(ref mut queue) = lock {
                            request = queue.pop_front();
                        }
                    }

                    if let Some(request) = request {
                        request();
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            });
        });
        thread_pool
    }

    pub fn enqueue(&self, task: impl FnOnce() + Send + 'static) {
        self.queue.lock().unwrap().push_back(Box::new(task));
    }
}
