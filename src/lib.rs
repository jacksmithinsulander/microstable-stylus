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
}

sol_interface! {
    interface IOracle {
        function latest_answer() external view returns (uint);
    }

    interface IErc20 {
        function transfer_from(address from, address to, uint256 value) external returns (bool);
        function transfer(address to, uint256 value) external returns (bool);
        function burn(address from, uint256 amount) external;
        function mint(address from, uint256 amount) external;
    }
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
    pub fn mint(&mut self, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            self.erc20.mint(self.vm().msg_sender(), amount);
            Ok(())
        } else {
            Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }

    pub fn burn(&mut self, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            self.erc20.burn(self.vm().msg_sender(), amount);
            Ok(())
        } else {
            Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }
}

#[public]
impl Manager {
    pub fn deposit(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let weth_instance = IErc20::new(self.weth.get());
        weth_instance.transfer_from(self, self.vm().msg_sender(), self.vm().contract_address(), amount)?;


        todo!()
    }
}
