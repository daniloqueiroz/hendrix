use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;

use crate::kprintln;
use crate::runtime::task::{Task, TaskId};

struct TaskWaker {
    task_id: TaskId,
    ready_tasks: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, ready_tasks: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            ready_tasks,
        }))
    }

    fn wake_task(&self) {
        self.ready_tasks
            .push(self.task_id)
            .expect("executor task_queue is full")
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub struct EventLoopExecutor {
    tasks: BTreeMap<TaskId, Task>,
    ready_tasks: Arc<ArrayQueue<TaskId>>,
    // TODO cache wakers so we optmize memory usage
}

impl EventLoopExecutor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            // TODO make the queue size a parameter
            ready_tasks: Arc::new(ArrayQueue::new(100)),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let id = task.id;
        self.tasks.insert(id, task);
        self.ready_tasks
            .push(id)
            .expect("executor task_queue is full");
    }

    pub fn wrap_future(&mut self, future: impl Future<Output = ()> + 'static) {
        self.spawn(Task::new(future))
    }

    /// Run the event loop until no there's no more task left to be executed.
    pub fn run<F>(&mut self, halt_func: F)
    where
        F: Fn() -> (),
    {
        while !self.tasks.is_empty() {
            if let Some(selected_task_id) = self.ready_tasks.pop() {
                let waker = TaskWaker::new(selected_task_id, self.ready_tasks.clone());
                let ctx = &mut Context::from_waker(&waker);
                let poll_result = self
                    .tasks
                    .get_mut(&selected_task_id)
                    .map(|task| task.poll(ctx))
                    .unwrap();
                match poll_result {
                    Poll::Ready(_) => {
                        self.tasks.remove(&selected_task_id);
                    }
                    Poll::Pending => halt_func(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::future::Future;
    use core::pin::Pin;
    use core::sync::atomic::{AtomicBool, Ordering};
    use core::task::{Context, Poll};

    use crate::runtime::executor::EventLoopExecutor;

    static IS_FUTURE_COMPLETE: AtomicBool = AtomicBool::new(false);

    struct TestableFuture {
        event_loop: *mut EventLoopExecutor,
        first_call: bool,
    }

    impl Future for TestableFuture {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match self.first_call {
                true => unsafe {
                    let waker = cx.waker().clone();
                    self.first_call = false;
                    self.event_loop
                        .as_mut()
                        .unwrap()
                        .wrap_future(async { waker.wake() });
                    Poll::Pending
                },
                false => {
                    IS_FUTURE_COMPLETE.store(true, Ordering::Relaxed);
                    Poll::Ready(())
                }
            }
        }
    }

    #[test_case]
    fn test_execute_simple_task() {
        let mut event_loop = EventLoopExecutor::new();

        IS_FUTURE_COMPLETE.store(false, Ordering::Relaxed);
        let future = async { IS_FUTURE_COMPLETE.store(true, Ordering::Relaxed) };

        event_loop.wrap_future(future);
        event_loop.run();

        assert!(IS_FUTURE_COMPLETE.load(Ordering::Relaxed))
    }

    #[test_case]
    fn test_execute_multiple_task() {
        let mut event_loop = EventLoopExecutor::new();
        IS_FUTURE_COMPLETE.store(false, Ordering::Relaxed);
        let future = TestableFuture {
            event_loop: &mut event_loop,
            first_call: true,
        };
        event_loop.wrap_future(future);
        event_loop.run();

        assert!(IS_FUTURE_COMPLETE.load(Ordering::Relaxed))
    }
}
