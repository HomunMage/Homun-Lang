/// homunc — Homun to Rust compiler
///
/// Entry point delegated to main_hom::main(), which is compiled from
/// src/main.hom via build.rs (homunc --module) and wired into lib.rs.
fn main() {
    homunc::main_hom::main();
}
