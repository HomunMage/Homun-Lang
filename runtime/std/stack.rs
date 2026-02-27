// ============================================================
// Homun Stack Library — included by ext.rs
// ============================================================

pub fn stack_new<T>() -> Vec<T> {
    Vec::new()
}

pub fn stack_push<T>(s: &mut Vec<T>, item: T) {
    s.push(item);
}

pub fn stack_pop<T>(s: &mut Vec<T>) -> Option<T> {
    s.pop()
}

pub fn stack_peek<T: Clone>(s: &Vec<T>) -> Option<T> {
    s.last().cloned()
}

pub fn stack_is_empty<T>(s: &Vec<T>) -> bool {
    s.is_empty()
}
