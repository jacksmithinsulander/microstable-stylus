// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::B256;
use rustmate::tokens::erc20::{ERC20Params, ERC20};
use rustmate::tokens::erc20;
use stylus_sdk::{alloy_primitives::U256, msg, prelude::*};


pub struct MicroParams;

impl erc20::ERC20Params for MicroParams {
    const NAME: &'static str = "Shafu USD";
    const SYMBOL: &'static str = "shUSD";
    const DECIMALS: u8 = 18;
    const INITIAL_CHAIN_ID: u64 = 1;
    const INITIAL_DOMAIN_SEPARATOR: alloy_primitives::B256 = B256::ZERO;
}

sol_storage! {
    #[entrypoint]
    pub struct ShUSD {
        #[borrow]
        erc20::ERC20<MicroParams> erc20;
        address manager;
    }

    // pub struct Manager {

    // }
}

#[external]
#[inherit(erc20::ERC20<MicroParams>)]
impl ShUSD {
    pub fn mint(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20.mint(msg::sender(), amount);
        Ok(())
    }

    pub fn burn(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20.burn(msg::sender(), amount);
        Ok(())
    }
}

