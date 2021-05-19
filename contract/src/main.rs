#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]

use casper_contract::{
    contract_api::{account, runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::NamedKeys,
    ApiError, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Parameter, URef, U512,
};

use std::convert::TryInto;

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

fn _set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

#[no_mangle]
pub extern "C" fn my_balance() {
    let balance = system::get_balance().unwrap_or_revert();
    runtime::put_key("caller_balance", storage::new_uref(balance).into());
}

#[no_mangle]
pub extern "C" fn get_contract_balance() {
    let local_purse: URef = get_key("contract_purse");
    let balance = system::get_purse_balance(local_purse).unwrap_or_revert();
    runtime::put_key("contract_balance", storage::new_uref(balance).into());
}

#[no_mangle]
pub extern "C" fn transfer_to() {
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U512 = runtime::get_named_arg("amount");
    let main_purse = account::get_main_purse();
    system::transfer_from_purse_to_account(main_purse, recipient, amount, None).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn transfer_to_short() {
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U512 = runtime::get_named_arg("amount");
    system::transfer_to_account(recipient, amount, None).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn pay_contract() {
    let amount: U512 = runtime::get_named_arg("amount");
    let main_purse = account::get_main_purse();
    let local_purse: URef = get_key("contract_purse");
    // runtime::revert(ApiError::User(9000));
    system::transfer_from_purse_to_purse(main_purse, local_purse, amount, None).unwrap_or_revert();
    runtime::revert(ApiError::User(9001));
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "transfer_to",
        vec![
            Parameter::new("recipient", AccountHash::cl_type()),
            Parameter::new("amount", CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "transfer_to_short",
        vec![
            Parameter::new("recipient", AccountHash::cl_type()),
            Parameter::new("amount", CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "pay_contract",
        vec![Parameter::new("amount", CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "my_balance",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "get_contract_balance",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        "contract_purse".to_string(),
        casper_types::Key::URef(system::create_purse()),
    );
    let (contract_hash, _) =
        storage::new_locked_contract(entry_points, Some(named_keys), None, None);
    runtime::put_key(
        "payment_contract_hash",
        storage::new_uref(contract_hash).into(),
    );
}
