// ============================================================
// Homun Dict Library — included by ext.rs
// ============================================================

pub fn keys<K: Clone, V>(d: &std::collections::HashMap<K, V>) -> Vec<K> {
    d.keys().cloned().collect()
}

pub fn values<K, V: Clone>(d: &std::collections::HashMap<K, V>) -> Vec<V> {
    d.values().cloned().collect()
}

pub fn entries<K: Clone, V: Clone>(d: &std::collections::HashMap<K, V>) -> Vec<(K, V)> {
    d.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}
