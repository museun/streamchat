#![allow(dead_code)]
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Queue<T> {
    data: VecDeque<T>,
    size: usize,
}

impl<T: Debug> Queue<T> {
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
