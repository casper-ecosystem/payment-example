#![no_main]
#![no_std]

use casper_contract::contract_api::{account, runtime, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::Key;
use casper_types::{runtime_args, ContractHash, RuntimeArgs};
mod constants;
use constants::{AMOUNT, DEPOSIT, DEPOSIT_CONTRACT_HASH, DEPOSIT_PURSE, DEPOSIT_RECIPIENT};

// This constant can be replaced easily with a named argument in the session code.
pub const MY_TRANSFER_PURSE: &str = "my_transfer_purse";

// Session code that executes in the callers context.
// This code will first try to get a purse stored under "my_deposit_purse", if not found it will create a new purse,
// and store it under the name. Session codes require an argument called `amount`,
// which is used as a limit to how many motes can be transferred from the `main_purse` of the account.
// NOTE: creating a new purse costs 2,5 cspr, consider storing and reusing them.
#[no_mangle]
pub extern "C" fn call() {
    let deposit_contract_hash: ContractHash = runtime::get_named_arg(DEPOSIT_CONTRACT_HASH);
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    // Get transport purse from the named keys, or if it doesn't exist, create and store a new one.
    let transport_purse = match runtime::get_key(MY_TRANSFER_PURSE) {
        Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
        None => {
            let new_purse = system::create_purse();
            runtime::put_key(MY_TRANSFER_PURSE, new_purse.into());
            new_purse
        }
    };
    let amount = runtime::get_named_arg(AMOUNT);
    // Transfer motes to the transport purse
    system::transfer_from_purse_to_purse(account::get_main_purse(), transport_purse, amount, None)
        .unwrap_or_revert();
    // Use the purse as an argument for the contract entrypoint.
    // NOTE: for the callee to be able to withdraw from this purse, the URef needs READ and WRITE access bytes.
    // NOTE_2: the callee side is able to store URefs with their access rights intact.
    let _: () = runtime::call_contract(
        deposit_contract_hash,
        DEPOSIT,
        runtime_args! {
            DEPOSIT_RECIPIENT => recipient,
            DEPOSIT_PURSE => transport_purse,
            AMOUNT => Some(amount)
        },
    );
}
