#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec;
use casper_contract::{
    contract_api::{
        runtime::{self, get_caller},
        storage,
        system::{self, create_purse, get_purse_balance, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
    Parameter, URef, U512,
};

mod constants;
use constants::{AMOUNT, COLLECT, DEPOSIT, DEPOSIT_PURSE, DEPOSIT_RECIPIENT};

// When depositing the contract checks if the recipient had a deposit purse stored previously.
// If not creates a new purse inside the contract for them (purse creation costs 2,5 cspr).
// Then the amount is transfered into the purse that is stored in the contract.
// Creating a purse in this process is a one time per user cost, since the purses are stored.
#[no_mangle]
pub extern "C" fn deposit() {
    let incoming_purse: URef = runtime::get_named_arg(DEPOSIT_PURSE);
    let recipient: Key = runtime::get_named_arg(DEPOSIT_RECIPIENT);
    let amount_u512: Option<U512> = runtime::get_named_arg(AMOUNT);
    let stored_purse =
        match runtime::get_key(&recipient.into_account().unwrap_or_revert().to_string()) {
            Some(purse_uref_key) => purse_uref_key.into_uref().unwrap_or_revert(),
            None => create_purse(),
        };
    let transfer_amount = match amount_u512 {
        Some(amount) => amount,
        None => get_purse_balance(stored_purse).unwrap_or_revert(),
    };
    transfer_from_purse_to_purse(incoming_purse, stored_purse, transfer_amount, None)
        .unwrap_or_revert();
    runtime::put_key(
        &recipient.into_account().unwrap_or_revert().to_string(),
        stored_purse.into(),
    );
}

// The `collect` entry_point checks whether there have been a deposit for the caller. If not then the call reverts with User(1) error.
// If a deposit purse is found, the desired amount (if contained within the purse) will be transfered directly to the callers account.
#[no_mangle]
pub extern "C" fn collect() {
    let recipient = get_caller();
    let escrow_purse = match runtime::get_key(&recipient.to_string()) {
        Some(purse_uref_key) => purse_uref_key.into_uref().unwrap_or_revert(),
        None => runtime::revert(ApiError::User(1)),
    };
    let amount_u512: Option<U512> = runtime::get_named_arg(AMOUNT);
    let transfer_amount = match amount_u512 {
        Some(amount) => amount,
        None => get_purse_balance(escrow_purse).unwrap_or_revert(),
    };
    system::transfer_from_purse_to_account(escrow_purse, recipient, transfer_amount, None)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        DEPOSIT,
        vec![
            Parameter::new(DEPOSIT_PURSE, URef::cl_type()),
            Parameter::new(DEPOSIT_RECIPIENT, Key::cl_type()),
            Parameter::new(AMOUNT, Option::<U512>::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        COLLECT,
        vec![Parameter::new(AMOUNT, Option::<U512>::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let (contract_hash, _version) = storage::new_contract(
        entry_points,
        None,
        Some("escrow_contract_package".to_string()),
        Some("escrow_contract_access_token".to_string()),
    );

    runtime::put_key("escrow_contract", contract_hash.into());

    // Added for the testing convinience.
    runtime::put_key(
        "escrow_contract_hash",
        storage::new_uref(contract_hash).into(),
    );
}
