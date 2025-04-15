// Only run this as a WASM if the export-abi feature is not set.
//#![cfg_attr(not(feature = "export-abi"), no_main)]
#![cfg_attr(target_arch = "wasm32", no_std, no_main)]

extern crate alloc;
pub mod token;
pub mod contracts;
pub mod micro;
