#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]

use casper_contract::contract_api::{runtime, system};
use casper_types::{account::AccountHash, runtime_args, ContractHash, RuntimeArgs, ApiError, U512};

#[no_mangle]
pub extern "C" fn call() {
    let payment_contract_hash: ContractHash = runtime::get_named_arg("payment_contract");
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U512 = runtime::call_contract(
        payment_contract_hash,
        "collect",
        runtime_args! {
            "recipient" => recipient,
        },
    );
    if amount != U512::from(100000000000000000u128) {
        runtime::revert(ApiError::User(33));
    }
}
