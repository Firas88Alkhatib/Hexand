use core::{ pin::Pin, task::{ Poll, Context } };
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{ task::AtomicWaker, Stream, StreamExt };
use ps2_mouse::{ Mouse, MouseState };

static MOUSE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

static MOUSE_QUEUE_SIZE: usize = 200;

pub fn add_packet(packet: u8) {
    if let Ok(queue) = MOUSE_QUEUE.try_get() {
        if let Err(_) = queue.push(packet) {
            println!("WARNING: mouse queue full; dropping mouse input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: Mouse queue not initialized");
    }
}

pub struct PacketStream {
    _private: (),
}

impl PacketStream {
    pub fn new() -> Self {
        MOUSE_QUEUE.try_init_once(|| ArrayQueue::new(MOUSE_QUEUE_SIZE)).expect("Mouse queue already initialized.");
        PacketStream { _private: () }
    }
}
impl Stream for PacketStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = MOUSE_QUEUE.try_get().expect("Mouse queue is not initialized");
        if let Some(packet) = queue.pop() {
            return Poll::Ready(Some(packet));
        }
        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(packet) => {
                WAKER.take();
                Poll::Ready(Some(packet))
            }
            None => { Poll::Pending }
        }
    }
}

pub async fn print_mouse_position() {
    fn handler(state: MouseState) {
        let pixels_moved_on_x = state.get_x();
        let pixels_moved_on_y = state.get_y();

        // TODO handle mouse inputs
        // The "state" can get how many pixels the mouse has moved from its current position
        // so somekind of global state is needed to store the last position of the mouse
        // the "state" also has functions for left and right mouse buttons up and down
    }
    let mut packets = PacketStream::new();
    let mut mouse = Mouse::new();
    mouse.set_on_complete(handler);
    println!("Mouse Task Started.");

    while let Some(packet) = packets.next().await {
        mouse.process_packet(packet);
    }
}