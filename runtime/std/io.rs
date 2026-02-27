// ============================================================
// Homun IO Library — included by ext.rs
// ============================================================

pub fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

pub fn write_file(path: &str, content: &str) {
    std::fs::write(path, content).unwrap();
}

pub fn eprint<T: std::fmt::Display>(msg: T) {
    eprintln!("{}", msg);
}

pub fn args() -> Vec<String> {
    std::env::args().collect()
}

pub fn exit(code: i32) {
    std::process::exit(code);
}
