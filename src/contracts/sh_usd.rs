// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{B256, Address};
use crate::token::erc20;
use stylus_sdk::{alloy_primitives::U256, prelude::*};
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256};
use alloy_sol_types::sol;

pub struct MicroParams;

impl erc20::ERC20Params for MicroParams {
    const NAME: &'static str = "Shafu USD";
    const SYMBOL: &'static str = "shUSD";
    const DECIMALS: u8 = 18;
    const INITIAL_CHAIN_ID: u64 = 1;
    const INITIAL_DOMAIN_SEPARATOR: alloy_primitives::B256 = B256::ZERO;
}

sol_storage! {
    //#[entrypoint]
    pub struct ShUSD {
        #[borrow]
        erc20::ERC20<MicroParams> erc20;
        address manager;
    }
}

sol! {
    error OnlyManagerCanCall();
}

#[derive(SolidityError)]
pub enum ShUSDErrors {
    OnlyManagerCanCall(OnlyManagerCanCall),
}

#[public]
#[inherit(erc20::ERC20<MicroParams>)]
impl ShUSD {
    pub fn mint(&mut self, to: Address, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            self.erc20.mint(to, amount);
            Ok(())
        } else {
            Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }

    pub fn burn(&mut self, from: Address, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            self.erc20.burn(from, amount);
            Ok(())
        } else {
            Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }
}

