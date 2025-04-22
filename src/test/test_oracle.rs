use alloc::vec::Vec;
use alloc::vec;
use stylus_sdk::{alloy_primitives::I256, prelude::*};

const PRICE: I256 = I256::from_limbs([170_147_173_700, 0, 0, 0]);

#[cfg_attr(feature = "test-oracle", stylus_sdk::prelude::entrypoint)]
#[storage]
pub struct TestOracle;

#[cfg_attr(feature = "test-oracle", stylus_sdk::prelude::public)]
#[cfg(feature = "test-oracle")]
impl TestOracle {
    pub fn latest_answer(&mut self) -> Result<I256, Vec<u8>> { Ok(PRICE) }
}
