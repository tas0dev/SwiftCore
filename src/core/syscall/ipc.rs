use crate::interrupt::spinlock::SpinLock;

use super::{EAGAIN, EINVAL};

const MAX_THREADS: usize = crate::task::ThreadQueue::MAX_THREADS;
const MAILBOX_CAP: usize = 64;

#[derive(Debug, Clone, Copy)]
struct Message {
    from: u64,
    value: u64,
}

impl Message {
    const fn empty() -> Self {
        Self { from: 0, value: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
struct Mailbox {
    head: usize,
    tail: usize,
    count: usize,
    buf: [Message; MAILBOX_CAP],
}

impl Mailbox {
    const fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            count: 0,
            buf: [Message::empty(); MAILBOX_CAP],
        }
    }

    fn push(&mut self, msg: Message) -> Result<(), ()> {
        if self.count >= MAILBOX_CAP {
            return Err(());
        }
        self.buf[self.tail] = msg;
        self.tail = (self.tail + 1) % MAILBOX_CAP;
        self.count += 1;
        Ok(())
    }

    fn pop(&mut self) -> Option<Message> {
        if self.count == 0 {
            return None;
        }
        let msg = self.buf[self.head];
        self.head = (self.head + 1) % MAILBOX_CAP;
        self.count -= 1;
        Some(msg)
    }
}

static MAILBOXES: SpinLock<[Mailbox; MAX_THREADS]> = SpinLock::new([Mailbox::new(); MAX_THREADS]);

/// IPC送信
pub fn send(dest_thread_id: u64, value: u64) -> u64 {
    if dest_thread_id == 0 {
        return EINVAL;
    }

    let sender = match crate::task::current_thread_id() {
        Some(id) => id.as_u64(),
        None => return EINVAL,
    };

    let idx = dest_thread_id.saturating_sub(1) as usize;
    if idx >= MAX_THREADS {
        return EINVAL;
    }

    let mut boxes = MAILBOXES.lock();
    if boxes[idx]
        .push(Message {
            from: sender,
            value,
        })
        .is_err()
    {
        return EAGAIN;
    }

    0
}

/// IPC受信
pub fn recv(sender_ptr: u64) -> u64 {
    let receiver = match crate::task::current_thread_id() {
        Some(id) => id.as_u64(),
        None => return EINVAL,
    };

    let idx = receiver.saturating_sub(1) as usize;
    if idx >= MAX_THREADS {
        return EINVAL;
    }

    let mut boxes = MAILBOXES.lock();
    let msg = match boxes[idx].pop() {
        Some(msg) => msg,
        None => return EAGAIN,
    };

    if sender_ptr != 0 {
        unsafe {
            (sender_ptr as *mut u64).write_volatile(msg.from);
        }
    }

    msg.value
}
