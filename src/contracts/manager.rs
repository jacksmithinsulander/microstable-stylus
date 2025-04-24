use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256, StorageBool};
use stylus_sdk::{alloy_primitives::U256, prelude::*, call::RawCall, alloy_sol_types::{sol, SolCall}};
use crate::alloc::string::ToString;
use core::str::FromStr;
use crate::contracts::calls;

const MIN_COLLAT_RATIO: u128 = 1_500_000_000_000_000_000; // 1.5e18

#[cfg_attr(feature = "manager", stylus_sdk::prelude::entrypoint)]
#[storage]
pub struct Manager {
    sh_usd: StorageAddress,
    weth: StorageAddress,
    oracle: StorageAddress,
    address_2deposit: StorageMap<Address, StorageU256>,
    address_2minted: StorageMap<Address, StorageU256>,
    is_initialized: StorageBool
}

#[cfg_attr(feature = "manager", stylus_sdk::prelude::public)]
#[cfg(feature = "manager")]
impl Manager {
    pub fn init(&mut self, weth_address: Address, oracle_address: Address, sh_usd_address: Address) {
        if self.is_initialized.get() {
            return;
        }
        self.weth.set(weth_address);
        self.oracle.set(oracle_address);
        self.sh_usd.set(sh_usd_address);
        self.is_initialized.set(true);
    }

    pub fn deposit(&mut self, amount: U256) {
        let sender = self.vm().msg_sender();
        let this = self.vm().contract_address();

        //unsafe { 
            //let _ = &RawCall::new().call(self.weth.get(), &transferFromCall {
                //from: sender,
                //to: this,
                //value: amount,
            //}.abi_encode());
        //};
        let _ = calls::transfer_from_call(self.weth.get(), sender, this, amount);

        let previus_balance = self.address_2deposit.get(sender);
        self.address_2deposit.insert(sender, previus_balance + amount);
    }

    pub fn burn(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2minted.get(sender);
        self.address_2minted.insert(sender, previous_balance - amount);
        //unsafe { 
            //let _ = &RawCall::new().call(self.sh_usd.get(), &burnCall {
                //from: sender,
                //amount: amount,
            //}.abi_encode());
        //}
        let _ = calls::burn_call(self.sh_usd.get(), sender, amount);
        Ok(())
    }

    pub fn mint(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2minted.get(sender);
        self.address_2minted.insert(sender, previous_balance + amount);
        match self.collat_ratio(sender) {
            Ok(result) => {
                if result < U256::from(MIN_COLLAT_RATIO) {
                    return Err(b"undercollateralized".to_vec());
                } else {
                    //unsafe { 
                        //&RawCall::new().call(self.sh_usd.get(), &mintCall {
                            //from: sender,
                            //amount: amount,
                        //}.abi_encode())
                    //};
                    let _ = calls::mint_call(self.sh_usd.get(), sender, amount);
                    Ok(())
                }
            },
            Err(e) => return Err(e) 
        }
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_deposit = self.address_2deposit.get(sender);
        self.address_2deposit.insert(sender, previous_deposit - amount);
        match self.collat_ratio(sender) {
            Ok(result) => {
                if result < U256::from(MIN_COLLAT_RATIO) {
                    return Err(b"Undercollateralized".to_vec());
                } else {
                    // unsafe { 
                        // let _ = &RawCall::new().call(self.weth.get(), &transferCall {
                            // to: sender,
                            // value: amount,
                        // }.abi_encode());
                    // }
                    let _ = calls::transfer_call(self.weth.get(), sender, amount);
                    Ok(())
                }
            },
            Err(e) => return Err(e)
        }
    }

    pub fn liquidate(&mut self, user: Address) -> Result<(), Vec<u8>> {
        match self.collat_ratio(user) {
            Ok(result) => {
                if result >= U256::from(MIN_COLLAT_RATIO) {
                    return Err(b"Not undercollateralized".to_vec());
                } else {
                    let sender = self.vm().msg_sender();
                    let amount_minted = self.address_2minted.get(user);
                    match calls::burn_call(self.sh_usd.get(), user, amount_minted) {
                        Ok(_) => {
                            let amount_deposited = self.address_2deposit.get(user);
                            let _ = calls::transfer_call(self.weth.get(), sender, amount_deposited);
                            //weth_instance.transfer(&mut *self, sender, amount_deposited);
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
        let deposited = self.address_2deposit.get(user);
        match calls::latest_answer_call(self.oracle.get()) {
            Ok(price) => {
                let value = deposited * (U256::from_str(&price.to_string()).unwrap() * U256::from(1e10 as u64));
                let value_scaled = value / U256::from(1e18 as u64);
                Ok(value_scaled / minted)
            }
            Err(e) => Err(e)
        }
    }
}
