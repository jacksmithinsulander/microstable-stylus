// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(target_arch = "wasm32", no_std, no_main)]

extern crate alloc;
pub mod token;
pub mod contracts;
pub mod micro;

#[cfg(all(
    target_arch = "wasm32",
    not(any(
        feature = "manager",
        feature = "sh-usd"
    ))
))]
compile_error!("one of the contract-* features must be enabled!");