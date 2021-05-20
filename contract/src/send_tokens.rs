#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]

use casper_contract::contract_api::{account, runtime, system};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, U512};

#[no_mangle]
pub extern "C" fn call() {
    let payment_contract_hash: ContractHash = runtime::get_named_arg("payment_contract");

    let transport_purse = system::create_purse();
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        transport_purse,
        U512::from(100000000000000000u128),
        None,
    )
    .unwrap();

    let _: () = runtime::call_contract(
        payment_contract_hash,
        "deposit",
        runtime_args! {
            "purse" => transport_purse
        },
    );
}
