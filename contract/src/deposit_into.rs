#![no_main]
#![no_std]

use casper_contract::contract_api::{account, runtime, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{runtime_args, ContractHash, RuntimeArgs};
use casper_types::{Key, URef};
mod constants;
use constants::{AMOUNT, DEPOSIT_RECIPIENT, ESCROW_CONTRACT_HASH, GET_DEPOSIT_PURSE};

// Session code that executes in the callers context.
// In this design we use a getter function to fetch a purse from the contract to deposit into.
// Session code NEEDS an argument called `amount`,
// which is used as a limit to how many motes can be extracted from the `main_purse` of the user.
// It might be important for you to migtigate the creation of new purses as it costs 2,5 CSPR.
#[no_mangle]
pub extern "C" fn call() {
    let escrow_contract_hash: ContractHash = runtime::get_named_arg(ESCROW_CONTRACT_HASH);
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    let amount_u512 = runtime::get_named_arg(AMOUNT);
    let deposit_purse: URef = runtime::call_contract(
        escrow_contract_hash,
        GET_DEPOSIT_PURSE,
        runtime_args! {
            DEPOSIT_RECIPIENT => recipient
        },
    );
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        deposit_purse,
        amount_u512,
        None,
    )
    .unwrap_or_revert();
}
