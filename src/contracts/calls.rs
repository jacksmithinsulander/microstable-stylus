use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256, StorageBool};
use stylus_sdk::{alloy_primitives::I256, prelude::*, call::RawCall, alloy_sol_types::{sol, SolCall}};
use crate::alloc::string::ToString;
use core::str::FromStr;

sol! {
    function latestAnswer() external view returns (int);
    function transferFrom(address from, address to, uint256 value) external returns (bool);
    function transfer(address to, uint256 value) external returns (bool);
    function burn(address from, uint256 amount) external;
    function mint(address from, uint256 amount) external;
}

sol!("./src/contracts/AggregatorV3Interface.sol");

pub fn latest_answer_call(oracle: Address) -> Result<I256, Vec<u8>> {
    match I256::try_from_be_slice(unsafe { &RawCall::new().call(oracle, &latestAnswerCall {}.abi_encode()).unwrap()}) {
        Some(res) => Ok(res),
        None => Err(b"Couldnt call".to_vec())
    }
}