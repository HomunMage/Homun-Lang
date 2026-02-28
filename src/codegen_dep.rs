// codegen_dep.rs — Type aliases and accessors for codegen.hom.
//
// Importing this file via `use codegen_dep` in codegen.hom sets has_rs_dep=true
// in the homunc sema checker, disabling undefined-reference checks for dep/*
// functions (scope_*, stmt_kind, expr_kind, codegen_type, etc.) and for
// runtime functions (join, push, etc.) that are available at include!() time
// in lib.rs but unknown to the homunc static checker.
//
// Type aliases:
//   RsContent = HashMap<String, String>   — resolved .rs file content map
//   HomFiles  = HashSet<String>           — resolved .hom dependency names
//
// Accessor helpers (owned-value signatures for .hom interop):
//   rs_content_get(map, key)  -> Option<String>
//   hom_files_contains(set, key) -> bool

pub type RsContent = std::collections::HashMap<String, String>;
pub type HomFiles = std::collections::HashSet<String>;

/// Look up a key in the rs_content map.  Returns None if absent.
pub fn rs_content_get(map: RsContent, key: String) -> Option<String> {
    map.get(&key).cloned()
}

/// Return true if the hom_files set contains the given key.
pub fn hom_files_contains(set: HomFiles, key: String) -> bool {
    set.contains(&key)
}
