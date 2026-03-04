use super::{
    Task, 
    TaskId
};
use alloc::{
    collections::BTreeMap,
    sync::Arc,
    task::Wake,
};
use core::task::{
    Context, 
    Poll, 
    Waker,
};
use crossbeam_queue::ArrayQueue;



// a custom waker that knows which task to wake
// and which queue to push it back into
struct TaskWaker {
    task_id: TaskId,                      // id of the task this waker belongs to
    task_queue: Arc<ArrayQueue<TaskId>>,  // shared queue of ready-to-run tasks
}

impl TaskWaker {
    // create a new waker instance for a specific task
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    // push the associated task back into the queue
    // so the executor will poll it again later
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

// implement the Wake trait so this can be used
// by rust's async system as a proper waker
impl Wake for TaskWaker {
    // when the task is woken, re-queue it for execution
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
}

// the main executor that drives all async tasks
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,         // all active tasks indexed by id
    task_queue: Arc<ArrayQueue<TaskId>>,  // queue of tasks that are ready to run
    waker_cache: BTreeMap<TaskId, Waker>, // cache of wakers to avoid recreating them
}

impl Executor {
    // create a new, empty executor with a fixed-size task queue
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)), // capacity = 100 tasks
            waker_cache: BTreeMap::new(),
        }
    }

    // start the executor loop
    // this never returns (!), since it runs forever
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks(); // poll all ready tasks
            self.sleep_if_idle();   // halt cpu if there's nothing to do
        }
    }

    // put the cpu to sleep if no tasks are ready
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        if self.task_queue.is_empty() {
            // enable interrupts and halt the cpu until the next one fires
            enable_and_hlt();
        } else {
            // if tasks are waiting, just make sure interrupts are enabled
            interrupts::enable();
        }
    }

    // add a new task to the executor
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        // ensure we do not overwrite an existing task
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same id already in tasks");
        }

        // push the new task into the ready queue so it gets polled
        self.task_queue.push(task_id).expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        // destructure 'self' to avoid borrow checker issues
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // process all tasks currently marked as ready
        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task may have already completed
            };

            // get or create a cached waker for this task
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            // create a context from the waker for polling
            let mut context = Context::from_waker(waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task completed -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {
                    // task is not ready yet, it will be re-queued by its waker
                }
            }
        } // while let Some(task_id)
    } // fn run_ready_tasks
} // impl Executor