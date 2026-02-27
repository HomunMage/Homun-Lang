// ============================================================
// Homun Runtime — heap.rs: Priority Queue (min-heap)
// Part B1 — stdlib, no external crates required.
//
// Usage in .hom:
//   use heap
//
//   h := heap_new()
//   heap_push(h, 5, "node_a")    // priority, item
//   heap_push(h, 2, "node_b")
//   heap_pop(h)                   // discards lowest-priority item
//   n := heap_len(h)
//   empty := heap_is_empty(h)
//
// Implementation note:
//   Uses Rc<RefCell<BinaryHeap<...>>> so that Homun's clone-based
//   calling convention (every variable argument becomes arg.clone())
//   still refers to the SAME underlying heap. Rc::clone() is a
//   cheap reference-count increment, not a deep copy, so all
//   "copies" of a Heap value share one BinaryHeap.
//
//   BinaryHeap wrapped with Reverse<i64> gives min-heap semantics
//   (smallest priority value is popped first).
// ============================================================

use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::rc::Rc;

/// Priority queue: min-heap keyed by i64, storing String items.
/// Rc<RefCell<...>> allows .hom's clone-based calling convention to
/// mutate through all handles that refer to the same heap.
pub type Heap = Rc<RefCell<BinaryHeap<(Reverse<i64>, String)>>>;

/// Create a new empty min-heap.
pub fn heap_new() -> Heap {
    Rc::new(RefCell::new(BinaryHeap::new()))
}

/// Push `item` onto the heap with the given `priority`.
/// Items with lower priority values are popped first.
/// Accepts `&str` so that .hom string literals (which codegen emits as &str)
/// and String variables (which Deref to &str) both work without conversion.
pub fn heap_push(h: Heap, priority: i64, item: &str) {
    h.borrow_mut().push((Reverse(priority), item.to_string()));
}

/// Pop and return the `(priority, item)` pair with the lowest priority.
/// Returns `None` if the heap is empty.
pub fn heap_pop(h: Heap) -> Option<(i64, String)> {
    h.borrow_mut().pop().map(|(Reverse(p), s)| (p, s))
}

/// Return the number of items in the heap (i32 for .hom int compatibility).
pub fn heap_len(h: Heap) -> i32 {
    h.borrow().len() as i32
}

/// Return `true` if the heap contains no items.
pub fn heap_is_empty(h: Heap) -> bool {
    h.borrow().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── heap_new ────────────────────────────────────────────
    #[test]
    fn test_heap_new_is_empty() {
        let h = heap_new();
        assert!(heap_is_empty(h.clone()));
        assert_eq!(heap_len(h.clone()), 0);
    }

    // ── heap_push / heap_len ─────────────────────────────────
    #[test]
    fn test_heap_push_increases_len() {
        let h = heap_new();
        heap_push(h.clone(), 10, "a".to_string());
        assert_eq!(heap_len(h.clone()), 1);
        heap_push(h.clone(), 5, "b".to_string());
        assert_eq!(heap_len(h.clone()), 2);
    }

    #[test]
    fn test_heap_is_empty_after_push() {
        let h = heap_new();
        heap_push(h.clone(), 1, "x".to_string());
        assert!(!heap_is_empty(h.clone()));
    }

    // ── heap_pop ────────────────────────────────────────────
    #[test]
    fn test_heap_pop_empty_returns_none() {
        let h = heap_new();
        assert_eq!(heap_pop(h.clone()), None);
    }

    #[test]
    fn test_heap_pop_single_item() {
        let h = heap_new();
        heap_push(h.clone(), 42, "only".to_string());
        assert_eq!(heap_pop(h.clone()), Some((42, "only".to_string())));
        assert!(heap_is_empty(h.clone()));
    }

    #[test]
    fn test_heap_min_order_two_items() {
        let h = heap_new();
        heap_push(h.clone(), 10, "high".to_string());
        heap_push(h.clone(), 2, "low".to_string());
        let (p, item) = heap_pop(h.clone()).unwrap();
        assert_eq!(p, 2);
        assert_eq!(item, "low");
    }

    #[test]
    fn test_heap_min_order_many_items() {
        let h = heap_new();
        heap_push(h.clone(), 5, "five".to_string());
        heap_push(h.clone(), 1, "one".to_string());
        heap_push(h.clone(), 3, "three".to_string());
        heap_push(h.clone(), 2, "two".to_string());
        heap_push(h.clone(), 4, "four".to_string());

        let mut pops: Vec<i64> = Vec::new();
        while let Some((p, _)) = heap_pop(h.clone()) {
            pops.push(p);
        }
        assert_eq!(pops, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_heap_pop_decreases_len() {
        let h = heap_new();
        heap_push(h.clone(), 1, "a".to_string());
        heap_push(h.clone(), 2, "b".to_string());
        heap_pop(h.clone());
        assert_eq!(heap_len(h.clone()), 1);
        heap_pop(h.clone());
        assert_eq!(heap_len(h.clone()), 0);
        assert!(heap_is_empty(h.clone()));
    }

    // ── negative and zero priorities ────────────────────────
    #[test]
    fn test_heap_negative_priorities() {
        let h = heap_new();
        heap_push(h.clone(), -5, "neg5".to_string());
        heap_push(h.clone(), 0, "zero".to_string());
        heap_push(h.clone(), -1, "neg1".to_string());

        let (p1, _) = heap_pop(h.clone()).unwrap();
        let (p2, _) = heap_pop(h.clone()).unwrap();
        let (p3, _) = heap_pop(h.clone()).unwrap();
        assert_eq!(p1, -5);
        assert_eq!(p2, -1);
        assert_eq!(p3, 0);
    }

    #[test]
    fn test_heap_zero_priority() {
        let h = heap_new();
        heap_push(h.clone(), 0, "origin".to_string());
        assert_eq!(heap_pop(h.clone()), Some((0, "origin".to_string())));
    }

    // ── tie-breaking (same priority) ────────────────────────
    // BinaryHeap breaks ties by the second element (String lexicographic order,
    // reversed). We only check that all items are returned, not their order.
    #[test]
    fn test_heap_same_priority_all_returned() {
        let h = heap_new();
        heap_push(h.clone(), 1, "alpha".to_string());
        heap_push(h.clone(), 1, "beta".to_string());
        heap_push(h.clone(), 1, "gamma".to_string());

        let mut items: Vec<String> = Vec::new();
        while let Some((p, item)) = heap_pop(h.clone()) {
            assert_eq!(p, 1);
            items.push(item);
        }
        items.sort();
        assert_eq!(items, vec!["alpha", "beta", "gamma"]);
    }

    // ── A* typical usage simulation ──────────────────────────
    #[test]
    fn test_heap_astar_simulation() {
        // Simulate: frontier queue in A* where priority = f-score
        let frontier = heap_new();
        heap_push(frontier.clone(), 10, "A".to_string());
        heap_push(frontier.clone(), 7, "B".to_string());
        heap_push(frontier.clone(), 15, "C".to_string());
        heap_push(frontier.clone(), 3, "D".to_string());

        // D has lowest f-score; should come out first
        let (p, node) = heap_pop(frontier.clone()).unwrap();
        assert_eq!(p, 3);
        assert_eq!(node, "D");

        // Add a new node discovered from D
        heap_push(frontier.clone(), 5, "E".to_string());

        let (p2, node2) = heap_pop(frontier.clone()).unwrap();
        assert_eq!(p2, 5);
        assert_eq!(node2, "E");
    }

    // ── Rc clone semantics: all handles share one heap ──────
    #[test]
    fn test_rc_clone_shares_state() {
        let h1 = heap_new();
        let h2 = h1.clone(); // cheap Rc clone — same inner heap
        heap_push(h1.clone(), 7, "seven".to_string());
        // h2 sees the item pushed via h1
        assert_eq!(heap_len(h2.clone()), 1);
        assert_eq!(heap_pop(h2.clone()), Some((7, "seven".to_string())));
        // h1 also sees it gone
        assert!(heap_is_empty(h1.clone()));
    }
}
