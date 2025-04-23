// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(target_arch = "wasm32", no_std, no_main)]

extern crate alloc;
pub mod token;
pub mod contracts;
pub mod test;

#[cfg(all(target_arch = "wasm32", feature = "harness-stylus-interpreter"))]
#[link(wasm_import_module = "stylus_interpreter")]
extern "C" {
    #[allow(dead_code)]
    // It's easier to do this than to go through the work of a custom panic handler.
    pub fn die(ptr: *const u8, len: usize, rc: i32);
}

#[cfg(all(
    target_arch = "wasm32",
    not(any(
        feature = "manager",
        feature = "sh-usd",
        feature = "test-oracle",
        feature = "test-weth"
    ))
))]
compile_error!("one of the contract-* features must be enabled!");