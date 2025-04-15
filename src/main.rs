// #![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

// #[cfg(not(any(test, feature = "export-abi")))]
// // #[no_mangle]
// pub extern "C" fn main() {}

// #[cfg(feature = "export-abi")]
// fn main() {
    // stylus_hello_world::print_abi("MIT-OR-APACHE-2.0", "pragma solidity ^0.8.23;");
// }
#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

#[cfg(target_arch = "wasm32")]
pub use microstable::user_entrypoint;

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
fn main() {}