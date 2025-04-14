// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;
pub mod token;
pub mod contracts;

use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::call::MethodError;
use stylus_sdk::{alloy_primitives::U256, prelude::*};
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256};

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

const MIN_COLLAT_RATIO: u128 = 1_500_000_000_000_000_000; // 1.5e18

#[entrypoint]
#[storage]
pub struct Manager {
    sh_usd: contracts::sh_usd::ShUSD,
    weth: StorageAddress,
    oracle: StorageAddress,
    address_2deposit: StorageMap<Address, StorageU256>,
    address_2minted: StorageMap<Address, StorageU256>
}

#[public]
impl Manager {
    pub fn deposit(&mut self, amount: U256) {
        let weth_instance = IErc20::new(self.weth.get());
        let sender = self.vm().msg_sender();
        let this = self.vm().contract_address();
        let _ = weth_instance.transfer_from(&mut *self, sender, this, amount);
        let previus_balance = self.address_2deposit.get(sender);
        self.address_2deposit.insert(sender, previus_balance + amount);
    }

    pub fn burn(&mut self, amount: U256) -> Result<(), Vec<u8>>{
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2minted.get(sender);
        self.address_2minted.insert(sender, previous_balance - amount);
        match self.sh_usd.burn(sender, amount) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    pub fn mint(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2minted.get(sender);
        self.address_2minted.insert(sender, previous_balance + amount);
        match self.collat_ratio(sender) {
            Ok(result) => {
                if result < U256::from(MIN_COLLAT_RATIO) {
                    return Err(b"Undercollateralized".to_vec());
                } else {
                    match self.sh_usd.mint(sender, amount) {
                        Ok(_) => return Ok(()),
                        Err(e) => return Err(e.into())
                    }
                }
            },
            Err(e) => return Err(e) 
        }
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2deposit.get(sender);
        self.address_2minted.insert(sender, previous_balance - amount);
        match self.collat_ratio(sender) {
            Ok(result) => {
                if result < U256::from(MIN_COLLAT_RATIO) {
                    return Err(b"Undercollateralized".to_vec());
                } else {
                    let weth_instance = IErc20::new(self.weth.get());
                    let _ = weth_instance.transfer(self, sender, amount);
                    Ok(())
                }
            },
            Err(e) => return Err(e) 
        }
    }

    pub fn liquidate(&mut self, user: Address) -> Result<(), Vec<u8>> {
        match self.collat_ratio(user) {
            Ok(result) => {
                if result > U256::from(MIN_COLLAT_RATIO) {
                    return Err(b"Not Undercollateralized".to_vec());
                } else {
                    let weth_instance = IErc20::new(self.weth.get());
                    match self.sh_usd.burn(self.vm().msg_sender(), self.address_2minted.get(user)) {
                        Ok(_) => {
                            let sender = self.vm().msg_sender();
                            let amount_deposited = self.address_2deposit.get(user);
                            let _ = weth_instance.transfer(&mut *self, sender, amount_deposited);
                            self.address_2deposit.insert(user, U256::ZERO);
                            self.address_2minted.insert(user, U256::ZERO);
                            Ok(())
                        },
                        Err(e) => return Err(e.into())
                    }
                }
            },
            Err(e) => return Err(e) 
        }
    }

    pub fn collat_ratio(&self, user: Address) -> Result<U256, Vec<u8>> {
        let minted = self.address_2minted.get(user);
        if minted.is_zero() {
            return Ok(U256::MAX);
        }

        let oracle_instance = IOracle::new(self.oracle.get());
        let deposited = self.address_2deposit.get(user);
        let price = match oracle_instance.latest_answer(self) {
            Ok(p) => p,
            Err(e) => return Err(e.encode())
        };
        let price_scaled = price * U256::from(1e10 as u64);
        let value = deposited * price_scaled / U256::from(1e18 as u64);
        Ok(value / minted)
    }
}
