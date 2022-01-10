#![no_main]
#![no_std]

use casper_contract::contract_api::{account, runtime, system};
use casper_types::{runtime_args, RuntimeArgs, U512};
use casper_types::{ContractPackageHash, Key};
mod constants;
use constants::{DEPOSIT, DEPOSIT_PURSE, DEPOSIT_RECIPIENT};
#[no_mangle]
pub extern "C" fn call() {
    let escrow_contract_hash: ContractPackageHash =
        runtime::get_named_arg("escrow_contract_package");
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    let transport_purse = system::create_purse();
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        transport_purse,
        U512::from(100000000000000000u128),
        None,
    )
    .unwrap();

    let _: () = runtime::call_versioned_contract(
        escrow_contract_hash,
        None,
        DEPOSIT,
        runtime_args! {
            DEPOSIT_RECIPIENT => recipient,
            DEPOSIT_PURSE => transport_purse
        },
    );
}
