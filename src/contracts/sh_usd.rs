// Only run this as a WASM if the export-abi feature is not set.
extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;
use alloy_primitives::Address;
use crate::token::erc20;
use stylus_sdk::{alloy_primitives::U256, prelude::*};
use alloy_sol_types::sol;

#[cfg_attr(any(feature = "sh-usd"), stylus_sdk::prelude::entrypoint)]

pub struct MicroParams;

impl erc20::Erc20Params for MicroParams {
    const NAME: &'static str = "Shafu USD";
    const SYMBOL: &'static str = "shUSD";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    //#[entrypoint]
    pub struct ShUSD {
        #[borrow]
        erc20::Erc20<MicroParams> erc20;
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

#[cfg_attr(feature = "sh-usd", stylus_sdk::prelude::public)]
#[public]
#[inherit(erc20::Erc20<MicroParams>)]
impl ShUSD {
    pub fn init(&mut self, manager_address: Address) {
        self.manager.set(manager_address);
    }

    pub fn mint(&mut self, to: Address, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            let _ = self.erc20.mint(to, amount);
            Ok(())
        } else {
            Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }

    pub fn burn(&mut self, from: Address, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            let _ = self.erc20.burn(from, amount);
            Ok(())
        } else {
            Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }
}

