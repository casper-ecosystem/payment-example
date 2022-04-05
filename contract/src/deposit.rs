#![no_main]
#![no_std]

use casper_contract::contract_api::{account, runtime, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{runtime_args, RuntimeArgs};
use casper_types::{ContractPackageHash, Key};
mod constants;
use constants::{AMOUNT, DEPOSIT, DEPOSIT_PURSE, DEPOSIT_RECIPIENT};

// Session code that executes in the callers context.
// This code will try to get a purse stored under "my_escrow_purse", if not found it will create a new purse.

#[no_mangle]
pub extern "C" fn call() {
    let escrow_contract_hash: ContractPackageHash =
        runtime::get_named_arg("escrow_contract_package");
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    let transport_purse = match runtime::get_key("my_escrow_purse") {
        Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = system::create_purse();
            runtime::put_key("my_escrow_purse", new_purse.into());
            new_purse
        }
    };
    let amount_u512 = runtime::get_named_arg(AMOUNT);
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        transport_purse,
        amount_u512,
        None,
    )
    .unwrap_or_revert();

    let _: () = runtime::call_versioned_contract(
        escrow_contract_hash,
        None,
        DEPOSIT,
        runtime_args! {
            DEPOSIT_RECIPIENT => recipient,
            DEPOSIT_PURSE => transport_purse,
            AMOUNT => Some(amount_u512)
        },
    );
}
