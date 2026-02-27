#![recursion_limit = "512"]
// Crate-level lint allows for patterns pervasive across the 186K LOC codebase.
// These should be revisited and tightened incrementally over time.
#![allow(clippy::too_many_arguments)] // 92 functions with >7 params — compiler passes inherently need many context args
#![allow(clippy::type_complexity)] // 20 complex type signatures — compiler IR types are naturally nested
#![allow(clippy::needless_range_loop)] // 56 indexed loops — many are intentional for multi-array coordination
#![allow(clippy::enum_variant_names)] // 3 enum variants — naming matches the domain terminology

pub mod backend;
pub(crate) mod common;
pub mod driver;
pub(crate) mod frontend;
pub(crate) mod ir;
pub(crate) mod passes;

/// Shared entry point for all compiler binaries. Spawns the real work on a
/// thread with a large stack so deeply recursive C files don't overflow.
pub fn compiler_main() {
    const STACK_SIZE: usize = 64 * 1024 * 1024; // 64 MB
    let builder = std::thread::Builder::new().stack_size(STACK_SIZE);
    let handler = builder
        .spawn(|| {
            let args: Vec<String> = std::env::args().collect();
            let mut driver = driver::Driver::new();
            if driver.parse_cli_args(&args)? {
                return Ok(());
            }
            if !driver.has_input_files() {
                return Err("no input files".to_string());
            }
            driver.run()
        })
        .expect("failed to spawn main thread");

    match handler.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("ccc: error: {}", e);
            std::process::exit(1);
        }
        Err(e) => {
            if let Some(s) = e.downcast_ref::<&str>() {
                eprintln!("ccc: internal error: {}", s);
            } else if let Some(s) = e.downcast_ref::<String>() {
                eprintln!("ccc: internal error: {}", s);
            } else {
                eprintln!("ccc: internal error (thread panicked)");
            }
            std::process::exit(1);
        }
    }
}
