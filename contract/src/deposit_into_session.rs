#![no_main]
#![no_std]

use casper_contract::contract_api::{account, runtime, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{runtime_args, ContractHash, RuntimeArgs};
use casper_types::{Key, URef};
mod constants;
use constants::{AMOUNT, DEPOSIT_CONTRACT_HASH, DEPOSIT_RECIPIENT, GET_DEPOSIT_PURSE};

// Session code that executes in the callers context.
// In this design we use a getter function to fetch a purse from the contract to deposit into.
// Session code REQUIRES an argument to be passed called `amount`,
// Which is used as a limit to how many motes can be transferred from the `main_purse` of the account.
// NOTE: creating a new purse costs 2,5 cspr, consider storing and reusing them.
#[no_mangle]
pub extern "C" fn call() {
    let deposit_contract_hash: ContractHash = runtime::get_named_arg(DEPOSIT_CONTRACT_HASH);
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    let amount = runtime::get_named_arg(AMOUNT);
    // Calling the deposit contract to get a URef to the deposit purse associated with the recipient
    let deposit_purse: URef = runtime::call_contract(
        deposit_contract_hash,
        GET_DEPOSIT_PURSE,
        runtime_args! {
            DEPOSIT_RECIPIENT => recipient
        },
    );
    // We transfer the specified amount into the deposit purse of the recipient.
    // As long as this function call does not fail the transfer of motes happens,
    // and there is no need to transfer the purse URef back to the contract.
    system::transfer_from_purse_to_purse(account::get_main_purse(), deposit_purse, amount, None)
        .unwrap_or_revert();
}
