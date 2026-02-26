// ============================================================
// Homun Deque Library — included by ext.rs
// ============================================================

use std::collections::VecDeque;

pub fn deque_new<T>() -> VecDeque<T> {
    VecDeque::new()
}

pub fn deque_push_front<T>(d: &mut VecDeque<T>, item: T) {
    d.push_front(item);
}

pub fn deque_push_back<T>(d: &mut VecDeque<T>, item: T) {
    d.push_back(item);
}

pub fn deque_pop_front<T>(d: &mut VecDeque<T>) -> Option<T> {
    d.pop_front()
}

pub fn deque_pop_back<T>(d: &mut VecDeque<T>) -> Option<T> {
    d.pop_back()
}

pub fn deque_is_empty<T>(d: &VecDeque<T>) -> bool {
    d.is_empty()
}
