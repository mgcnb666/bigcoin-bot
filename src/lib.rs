// https://bigcoin.tech

use alloy::{
    primitives::{Address, address},
    sol,
};

pub mod add_starter;
pub mod claim;
pub mod initialize;
pub mod transfer;
pub mod print;

mod provider_ext;

const CONTRACT: Address = address!("0x09Ee83D8fA0f3F03f2aefad6a82353c1e5DE5705");
const TOKEN: Address = address!("0xDf70075737E9F96B078ab4461EeE3e055E061223");
pub const CHAIN_ID: u64 = 2741;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    BigcoinAbi,
    "bigcoin.abi.json"
}
