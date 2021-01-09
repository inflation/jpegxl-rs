#![allow(missing_docs)]
use std::ffi::c_void;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::parallel::*;

type Job = Box<dyn Fn(u64) + Send + 'static>;

struct CPointer(*mut c_void);

unsafe impl Send for CPointer {}

pub enum Message {
    NewJob(Job),
    Terminate,
}

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job(id as u64);
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ThreadPoolRunner {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPoolRunner {
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_workers);

        for id in 0..num_workers {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: Fn(u64) + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPoolRunner {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl JXLParallelRunner for ThreadPoolRunner {
    fn runner(&self) -> ParallelRunnerFn {
        unsafe extern "C" fn runner_func(
            runner_opaque: *mut c_void,
            jpegxl_opaque: *mut c_void,
            init_func: Option<InitFn>,
            run_func: Option<RunFn>,
            start_range: u32,
            end_range: u32,
        ) -> JxlParallelRetCode {
            let runner = (runner_opaque as *mut ThreadPoolRunner).as_ref().unwrap();
            let ret_code = init_func.unwrap()(jpegxl_opaque, runner.workers.len() as u64);
            if ret_code != 0 {
                return ret_code;
            };

            let chunk_size =
                ((end_range - start_range) as f64 / runner.workers.len() as f64).ceil() as usize;

            let ptr = CPointer(jpegxl_opaque);
            runner.execute(move |id| {
                println!("opaque ptr: {:#?}, thread id: {}", ptr.0, id);
                for i in start_range..end_range {
                    run_func.unwrap()(ptr.0, i, id);
                }
            });

            0
        }
        runner_func
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threadpool() {
        use crate::*;

        let runner = Box::new(ThreadPoolRunner::new(8));
        let mut dec: JXLDecoder<u8> = decoder_builder().parallel_runner(runner).build();

        let sample = std::fs::read("test/sample.jxl").unwrap();
        dec.decode(&sample).unwrap();
    }
}
