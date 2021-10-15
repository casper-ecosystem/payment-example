#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec;
use casper_contract::{
    contract_api::{
        runtime::{self, call_versioned_contract, get_caller},
        storage::{self, dictionary_get, dictionary_put, new_dictionary},
        system::{self, get_purse_balance},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, CLType, CLTyped, EntryPoint,
    EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter, RuntimeArgs, URef,
};

#[no_mangle]
pub extern "C" fn init() {
    new_dictionary("escrow").unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn deposit() {
    let purse: URef = runtime::get_named_arg("purse");
    let recipient: Key = runtime::get_named_arg("recipient");
    store_escrow_purse(recipient, purse);
}

#[no_mangle]
pub extern "C" fn collect() {
    let recipient = get_caller();
    let escrow_purse = get_escrow_purse(Key::Account(recipient));
    let escrow_balance = get_purse_balance(escrow_purse).unwrap_or_revert();
    system::transfer_from_purse_to_account(escrow_purse, recipient, escrow_balance, None)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn call() {
    let (contract_package_hash, _access_uref) = storage::create_contract_package_at_hash();

    let admin_group = storage::create_contract_user_group(
        contract_package_hash,
        "group_label",
        1,
        Default::default(),
    )
    .unwrap();

    runtime::put_key("group_uref", admin_group[0].into());

    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "init",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "deposit",
        vec![Parameter::new("purse", CLType::URef)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "collect",
        vec![Parameter::new("recipient", AccountHash::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let mut named_keys = NamedKeys::new();
    let purse = system::create_purse();
    named_keys.insert("contract_purse".to_string(), purse.into());

    // Added for the testing convinience.
    named_keys.insert(
        "contract_purse_wrapper".to_string(),
        storage::new_uref(purse).into(),
    );
    named_keys.insert(
        "payment_contract_package".to_string(),
        storage::new_uref(contract_package_hash).into(),
    );
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);

    runtime::put_key("payment_contract", contract_hash.into());

    // Added for the testing convinience.
    runtime::put_key(
        "payment_contract_hash",
        storage::new_uref(contract_hash).into(),
    );

    call_versioned_contract(contract_package_hash, None, "init", runtime_args! {})
}

fn get_escrow_purse(key: Key) -> URef {
    let dict_key = runtime::get_key("escrow").unwrap_or_revert();
    let dict_uref = dict_key.into_uref().unwrap_or_revert();
    dictionary_get(
        dict_uref,
        &key.into_account().unwrap_or_revert().to_string(),
    )
    .unwrap_or_revert()
    .unwrap_or_revert()
}

fn store_escrow_purse(key: Key, purse: URef) {
    let dict_key = runtime::get_key("escrow").unwrap_or_revert();
    let dict_uref = dict_key.into_uref().unwrap_or_revert();
    dictionary_put(
        dict_uref,
        &key.into_account().unwrap_or_revert().to_string(),
        purse,
    )
}
