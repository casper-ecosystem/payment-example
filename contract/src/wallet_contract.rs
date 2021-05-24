#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]

use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{CLTyped, CLValue, account::AccountHash};
use casper_types::Group;
use casper_types::{
    contracts::NamedKeys,
    runtime_args, ApiError, CLType, ContractHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter, PublicKey, RuntimeArgs, URef, U512,
};


#[no_mangle]
pub extern "C" fn deposit() {
    let purse: URef = runtime::get_named_arg("purse");
    let amount: U512 = system::get_purse_balance(purse).unwrap();
    let contract_purse: URef = runtime::get_key("contract_purse")
        .unwrap()
        .into_uref()
        .unwrap();
    system::transfer_from_purse_to_purse(purse, contract_purse, amount, None).unwrap();
}

#[no_mangle]
pub extern "C" fn collect() {
    // send all tokens from "contract_purse" to given account;
    let recipient: AccountHash = runtime::get_named_arg("recipient");

    let contract_purse: URef = runtime::get_key("contract_purse")
        .unwrap()
        .into_uref()
        .unwrap();
    let contract_balance = system::get_purse_balance(contract_purse).unwrap_or_revert();

    system::transfer_from_purse_to_account(
        contract_purse,
        recipient,
        contract_balance,
        None
    ).unwrap();
    runtime::ret(CLValue::from_t(contract_balance).unwrap_or_revert());
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
        "deposit",
        vec![Parameter::new("purse", CLType::URef)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "collect",
        vec![
            Parameter::new("recipient", AccountHash::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Groups(vec![Group::new("group_label")]),
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
}
