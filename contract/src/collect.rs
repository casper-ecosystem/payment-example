#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]

use casper_types::ContractPackageHash;
use casper_contract::contract_api::{runtime, system};
use casper_types::{account::AccountHash, runtime_args, ContractHash, RuntimeArgs, URef, U512};

#[no_mangle]
pub extern "C" fn call() {
    let payment_contract_hash: ContractPackageHash = runtime::get_named_arg("payment_contract_package");
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U512 = runtime::get_named_arg("amount");
    let purse: URef = runtime::call_versioned_contract(
        payment_contract_hash,
        None,
        "collect",
        runtime_args! {
            "amount" => amount,
        },
    );

    system::transfer_from_purse_to_account(
        purse,
        recipient,
        system::get_purse_balance(purse).unwrap(),
        None,
    )
    .unwrap();
}
