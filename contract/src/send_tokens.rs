#![no_main]

use casper_contract::contract_api::{account, runtime, system};
use casper_types::ContractPackageHash;
use casper_types::{runtime_args, RuntimeArgs, U512};

#[no_mangle]
pub extern "C" fn call() {
    let payment_contract_hash: ContractPackageHash = runtime::get_named_arg("payment_contract");

    let transport_purse = system::create_purse();
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        transport_purse,
        U512::from(100000000000000000u128),
        None,
    )
    .unwrap();

    let _: () = runtime::call_versioned_contract(
        payment_contract_hash,
        None,
        "deposit",
        runtime_args! {
            "purse" => transport_purse
        },
    );
}
