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

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue() {
        let mut queue = Queue::new(4);
        let results = std::iter::repeat(None)
            .take(4)
            .chain((0..13).map(Some))
            .collect::<Vec<_>>();

        assert_eq!(queue.is_empty(), true);
        assert_eq!(queue.size(), 4);

        for n in 0..13 {
            assert_eq!(queue.push(n), results[n])
        }

        assert_eq!(queue.len(), 4);

        let mut results = (9..).take(4).map(Some).collect::<Vec<_>>();
        results.push(None);

        for result in results {
            assert_eq!(queue.pop(), result)
        }

        assert_eq!(queue.len(), 0);
        assert_eq!(queue.is_empty(), true);
    }
}
