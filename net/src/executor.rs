use cronet_sys::*;
use crossbeam::channel::{self, Receiver, Sender};
use std::{ffi::c_void, thread::JoinHandle};

pub struct Executor {
    pub(crate) exec_ptr: Cronet_ExecutorPtr,
    sender_box: *mut Sender<Task>,
    join_handle: Option<JoinHandle<()>>,
}
unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

struct Task {
    command: Cronet_RunnablePtr,
}
unsafe impl Send for Task {}

impl Executor {
    pub fn new() -> Self {
        let (send, recv) = channel::unbounded();

        let join_handle = std::thread::spawn(move || exec_thread(recv));

        let sender_box: *mut Sender<Task> = Box::into_raw(Box::new(send));

        unsafe {
            let exec_ptr = Cronet_Executor_CreateWith(Some(execute));
            Cronet_Executor_SetClientContext(exec_ptr, sender_box as *mut c_void);

            Self {
                exec_ptr,
                sender_box,
                join_handle: Some(join_handle),
            }
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        unsafe {
            // First drop the Sender.
            // The destructor of Sender signals to the thread to exit.
            drop(Box::from_raw(self.sender_box));

            // Then wait for thread to exit
            // We use unwrap() because it is a bug if the join handle is not present.
            self.join_handle
                .take()
                .unwrap()
                .join()
                .expect("Panic in worker thread");

            // deallocate cronet executor
            Cronet_Executor_Destroy(self.exec_ptr);
        }
    }
}

unsafe extern "C" fn execute(self_: Cronet_ExecutorPtr, command: Cronet_RunnablePtr) {
    let sender_box = Cronet_Executor_GetClientContext(self_) as *mut Sender<Task>;
    if (*sender_box).send(Task { command }).is_err() {
        // thread has shut down
        Cronet_Runnable_Destroy(command);
    }
}

fn exec_thread(recv: Receiver<Task>) {
    while let Ok(Task { command }) = recv.recv() {
        unsafe {
            Cronet_Runnable_Run(command);
            Cronet_Runnable_Destroy(command);
        }
    }
    // when the sender is dropped, we exit the loop
}
