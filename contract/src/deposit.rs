#![no_main]
#![no_std]

use casper_contract::contract_api::{account, runtime, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::Key;
use casper_types::{runtime_args, ContractHash, RuntimeArgs};
mod constants;
use constants::{AMOUNT, DEPOSIT, DEPOSIT_PURSE, DEPOSIT_RECIPIENT, ESCROW_CONTRACT_HASH};

// This constant can be replaced easily with a named argument in the session code.
pub const MY_ESCROW_PURSE: &str = "my_escrow_purse";

// Session code that executes in the callers context.
// This code will try to get a purse stored under "my_escrow_purse", if not found it will create a new purse.
// Session code NEEDS an argument called `amount`,
// which is used as a limit to how many motes can be extracted from the `main_purse` of the user.
// It might be important for you to reuse purses as their creation costs 2,5 CSPR.
#[no_mangle]
pub extern "C" fn call() {
    let escrow_contract_hash: ContractHash = runtime::get_named_arg(ESCROW_CONTRACT_HASH);
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    let transport_purse = match runtime::get_key(MY_ESCROW_PURSE) {
        Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = system::create_purse();
            runtime::put_key(MY_ESCROW_PURSE, new_purse.into());
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

    let _: () = runtime::call_contract(
        escrow_contract_hash,
        DEPOSIT,
        runtime_args! {
            DEPOSIT_RECIPIENT => recipient,
            DEPOSIT_PURSE => transport_purse,
            AMOUNT => Some(amount_u512)
        },
    );
}
