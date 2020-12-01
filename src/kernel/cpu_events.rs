use core::pin::Pin;

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::{AtomicWaker, Context, Poll};

use crate::kprintln;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            kprintln!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake()
        }
    } else {
        panic!("Keyboard code received before CPU event queues be initialized")
    }
}

pub struct KeyboardStream {}

impl KeyboardStream {
    pub(crate) fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("Unable to initialize Keyboard event queue");
        KeyboardStream {}
    }
}

impl Stream for KeyboardStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<u8>> {
        if let Ok(queue) = SCANCODE_QUEUE.try_get() {
            WAKER.register(ctx.waker());
            match queue.pop() {
                Some(scancode) => Poll::Ready(Some(scancode)),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
