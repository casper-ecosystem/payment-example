#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]

use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::CLValue;
use casper_types::Group;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::NamedKeys,
    runtime_args, ApiError, CLType, CLTyped, ContractHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter, PublicKey, RuntimeArgs, URef, U512,
};

use std::convert::TryInto;

fn _get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
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
pub extern "C" fn buy_nft() {
    let purse: URef = runtime::get_named_arg("purse");
    let recipient: PublicKey = runtime::get_named_arg("recipient");
    let amount: U512 = system::get_purse_balance(purse).unwrap();

    // Check if the sent amount is correct.
    let nft_price = U512::from(1000000);
    if amount != nft_price {
        runtime::revert(ApiError::User(1));
    }

    // Accept the payment.
    let contract_purse: URef = runtime::get_key("contract_purse")
        .unwrap()
        .into_uref()
        .unwrap();
    system::transfer_from_purse_to_purse(purse, contract_purse, amount, None).unwrap();

    let nft_contract: ContractHash = runtime::get_key("nft_contract")
        .unwrap_or_revert()
        .into_hash()
        .unwrap()
        .into();

    let _: () = runtime::call_contract(
        nft_contract,
        "mint_one",
        runtime_args! {
            "recipient" => recipient,
            "token_uri" => String::from("QmWWQSuPMS6aXCbZKpEjPHPUZN2NjB3YrhJTHsV4X3vb2t")
        },
    );
}

#[no_mangle]
pub extern "C" fn collect() {
    // send all tokens from "contract_purse" to given account;
    let amount: U512 = runtime::get_named_arg("amount");
    let transport_purse = system::create_purse();
    let contract_purse: URef = runtime::get_key("contract_purse")
        .unwrap()
        .into_uref()
        .unwrap();
    let contract_balance = system::get_purse_balance(contract_purse).unwrap_or_revert();
    system::transfer_from_purse_to_purse(
        contract_purse,
        transport_purse,
        if amount > contract_balance {
            contract_balance
        } else {
            amount
        },
        None,
    )
    .unwrap_or_revert();
    runtime::ret(CLValue::from_t(transport_purse).unwrap_or_revert())
    // ...
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
            Parameter::new("amount", CLType::U512),
            Parameter::new("purse", CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Groups(vec![Group::new("group_label")]),
        EntryPointType::Contract,
    ));

    let mut named_keys = NamedKeys::new();
    let purse = system::create_purse();
    named_keys.insert("contract_purse".to_string(), purse.into());
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
    runtime::put_key(
        "payment_contract_hash",
        storage::new_uref(contract_hash).into(),
    );
}
