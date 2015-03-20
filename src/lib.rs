#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;
use std::mem;

struct Node<T> {
    ticket: AtomicUsize,
    data: Option<T>
}

impl<T> Node<T> {
    fn new(ticket: usize) -> Node<T> {
        Node {
            ticket: AtomicUsize::new(ticket),
            data: None
        }
    }
}

pub struct Queue<T> {
    buffer: Vec<Node<T>>,
    mask: usize,
    enqueue_index: AtomicUsize,
    dequeue_index: AtomicUsize
}

impl<T> Queue<T> {
    fn new(bound: usize) -> Queue<T> {
        assert!(bound >= 2);
        assert_eq!(bound & (bound - 1), 0);

        let mut buffer = Vec::with_capacity(bound);
        for i in 0..bound {
            buffer.push(Node::new(i));
        }

        Queue {
            buffer: buffer,
            mask: bound - 1,
            enqueue_index: AtomicUsize::new(0),
            dequeue_index: AtomicUsize::new(0)
        }
    }

    pub fn enqueue(&mut self, item: T) -> bool {
        let mut index = self.enqueue_index.load(Ordering::Relaxed);
        loop {
            let mut cell = &self.buffer[index & self.mask];
            let ticket = cell.ticket.load(Ordering::Acquire);
            if ticket == index {
                if index == self.enqueue_index.compare_and_swap(index, index + 1, Ordering::Relaxed) {
                    mem::replace(cell.data, Some(item));
                    cell.ticket.store(index + 1, Ordering::Relaxed);
                    return true;
                }
            } else if ticket < index {
                return false;
            } else {
                index = self.enqueue_index.load(Ordering::Relaxed);
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        let mut index = self.dequeue_index.load(Ordering::Relaxed);
        loop {
            let cell = &self.buffer[index & self.mask];
            let ticket = cell.ticket.load(Ordering::Acquire);
            if ticket == index + 1 {
                if index == self.dequeue_index.compare_and_swap(index, index + 1, Ordering::Relaxed) {
                    let data = cell.data.take();
                    cell.ticket.store(index + self.mask + 1, Ordering::Release);
                    return data;
                }
            } else if ticket < index + 1 {
                return None;
            } else {
                index = self.dequeue_index.load(Ordering::Relaxed);
            }
        }
    }
}
