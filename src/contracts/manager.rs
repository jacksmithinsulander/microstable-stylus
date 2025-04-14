// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;
pub mod token;

use alloc::vec::Vec;
use alloy_primitives::{B256, Address};
use token::erc20;
use stylus_sdk::{alloy_primitives::U256, prelude::*};
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256};
use alloy_sol_types::sol;


sol_interface! {
    interface IOracle {
        function latest_answer() external view returns (uint);
    }

    // interface IErc20 {
        // function transfer_from(address from, address to, uint256 value) external returns (bool);
        // function transfer(address to, uint256 value) external returns (bool);
        // function burn(address from, uint256 amount) external;
        // function mint(address from, uint256 amount) external;
    // }
}

#[storage]
#[entrypoint]
pub struct Manager {
    sh_usd: StorageAddress,
    weth: StorageAddress,
    oracle: StorageAddress,
    address_2deposit: StorageMap<Address, StorageU256>,
    address_2minted: StorageMap<Address, StorageU256>
}

#[public]
impl Manager {
    pub fn deposit(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let weth_instance = IErc20::new(self.weth.get());
        weth_instance.transfer_from(self, self.vm().msg_sender(), self.vm().contract_address(), amount)?;


        todo!()
    }
}
