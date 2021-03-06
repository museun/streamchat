#![allow(dead_code)]
use std::collections::VecDeque;

pub struct Queue<T> {
    data: VecDeque<T>,
    size: usize,
}

impl<T> Queue<T> {
    pub fn new(size: usize) -> Self {
        Queue {
            data: VecDeque::with_capacity(size),
            size,
        }
    }

    pub fn push(&mut self, element: T) -> Option<T> {
        let out = if self.data.len() == self.size {
            self.data.pop_front()
        } else {
            None
        };

        self.data.push_back(element);
        out
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Queue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue").field("size", &self.size).finish()
    }
}
