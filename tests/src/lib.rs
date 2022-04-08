use std::path::PathBuf;

use casper_engine_test_support::{InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST};

use casper_types::{account::AccountHash, runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};
use casper_types::{ContractHash, Key};
use utils::{deploy, fund_account, query, DeploySource};

mod utils;

pub struct PaymentContract {
    pub builder: InMemoryWasmTestBuilder,
    pub contract_hash: ContractHash,
    pub alice_account: AccountHash,
    pub bob_account: AccountHash,
    pub charlie_account: AccountHash,
}

impl PaymentContract {
    pub fn deploy() -> Self {
        // We create 3 accounts. "alice" will be the one who installs the contract.
        let alice_public_key: PublicKey =
            PublicKey::from(&SecretKey::ed25519_from_bytes([1u8; 32]).unwrap());
        let bob_public_key: PublicKey =
            PublicKey::from(&SecretKey::ed25519_from_bytes([2u8; 32]).unwrap());
        let charlie_public_key: PublicKey =
            PublicKey::from(&SecretKey::ed25519_from_bytes([3u8; 32]).unwrap());
        // Get addresses for participating accounts.
        let alice_account = AccountHash::from(&alice_public_key);
        let bob_account = AccountHash::from(&bob_public_key);
        let charlie_account = AccountHash::from(&charlie_public_key);

        // Set up the test framework and fund accounts
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder
            .exec(fund_account(&alice_account))
            .expect_success()
            .commit();
        builder
            .exec(fund_account(&bob_account))
            .expect_success()
            .commit();
        builder
            .exec(fund_account(&charlie_account))
            .expect_success()
            .commit();

        // install contract
        let code = PathBuf::from("deposit_contract.wasm");
        deploy(
            &mut builder,
            &alice_account,
            &DeploySource::Code(code),
            runtime_args! {},
            true,
            None,
        );

        // query the contracts hash from alice account storage
        let contract_hash = query(
            &builder,
            Key::Account(alice_account),
            &["deposit_contract_hash".to_string()],
        );

        Self {
            builder,
            contract_hash,
            alice_account,
            bob_account,
            charlie_account,
        }
    }
    /// Getter function for the balance of an account.
    fn get_balance(&self, account_key: &AccountHash) -> U512 {
        let account = self
            .builder
            .get_account(*account_key)
            .expect("should get genesis account");
        self.builder.get_purse_balance(account.main_purse())
    }

    /// Shorthand to get the balances of all 3 accounts in order.
    pub fn get_all_accounts_balance(&self) -> (U512, U512, U512) {
        (
            self.get_balance(&self.alice_account),
            self.get_balance(&self.bob_account),
            self.get_balance(&self.charlie_account),
        )
    }

    /// Function that handles the creation and execution of deploys.
    fn call(&mut self, caller: AccountHash, entry_point: &str, args: RuntimeArgs) {
        deploy(
            &mut self.builder,
            &caller,
            &DeploySource::ByContractHash {
                hash: self.contract_hash,
                entry_point: entry_point.to_string(),
            },
            args,
            true,
            None,
        );
    }

    /// Deploys the "deposit_session" with recipient and the hash of the "deposit_contract",
    /// that creates a purse and transfers `amount` number of motes into it,
    /// then transfers said purse to the deposit entry_point of the contract.
    pub fn deposit(&mut self, sender: AccountHash, recipient: Key, amount: U512) {
        let code = PathBuf::from("deposit_session.wasm");
        let args = runtime_args! {
            "deposit_contract_hash" => self.contract_hash,
            "recipient" => recipient,
            "amount" => amount
        };
        deploy(
            &mut self.builder,
            &sender,
            &DeploySource::Code(code),
            args,
            true,
            None,
        );
    }

    /// Deploy "deposit_into_session" that has the same arguments as "deposit_session", but instead of
    /// passing in a purse to the contract to do a transfer of motes, this session asks the contract
    /// for a purse and deposits motes into it.
    pub fn deposit_into(&mut self, sender: AccountHash, recipient: Key, amount: U512) {
        let code = PathBuf::from("deposit_into_session.wasm");
        let args = runtime_args! {
            "deposit_contract_hash" => self.contract_hash,
            "recipient" => recipient,
            "amount" => amount
        };
        deploy(
            &mut self.builder,
            &sender,
            &DeploySource::Code(code),
            args,
            true,
            None,
        );
    }

    /// Function that calls the `collect` endpoint on the deposit contract,
    /// that directly transfers the amount in the purse stored to the accounts hash to the account.
    pub fn collect(&mut self, recipient: AccountHash) {
        self.call(
            recipient,
            "collect",
            runtime_args! {"amount" => Option::<U512>::None},
        );
    }
}

#[test]
fn test_payment_and_collect() {
    // Setup example contract context
    let mut context = PaymentContract::deploy();

    // Print the balance of all 3 accounts

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(48500000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // alice deposits motes into the contract for charlie to withdraw
    context.deposit(
        context.alice_account,
        Key::Account(context.charlie_account),
        U512::from(10000000000000u64),
    );

    // look at balances again, alice money should be down by the deposited amount of 10000000000000,
    // and another 1500000000000 that is the contract deployment cost in the tests.
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // charlie collects his tokens from the deposit
    context.collect(context.charlie_account);

    // we verify that charlie indeed gained 10000000000000 motes
    // (-1500000000000 that was the cost of calling `collect`)
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.2, U512::from(58500000000000_u64));
}

#[test]
fn test_multiple_payment_and_single_collect() {
    // Setup example contract context
    let mut context = PaymentContract::deploy();

    // Default state (after deployment)
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(48500000000000_u64));
    assert_eq!(account_balances.1, U512::from(50000000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // alice makes a deposit for charlie
    context.deposit(
        context.alice_account,
        Key::Account(context.charlie_account),
        U512::from(10000000000000u64),
    );

    // bob also makes a deposit for charlie
    context.deposit(
        context.bob_account,
        Key::Account(context.charlie_account),
        U512::from(10000000000000u64),
    );

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.1, U512::from(38500000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // charlie collects the deposits
    context.collect(context.charlie_account);

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.1, U512::from(38500000000000_u64));
    assert_eq!(account_balances.2, U512::from(68500000000000_u64));
}

#[test]
fn test_multiple_payment_and_single_collect_deposit_into() {
    // Setup example contract context
    let mut context = PaymentContract::deploy();

    // Default state (after deployment)
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(48500000000000_u64));
    assert_eq!(account_balances.1, U512::from(50000000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // alice and bob make their deposits for charlie using "deposit_into_session"
    context.deposit_into(
        context.alice_account,
        Key::Account(context.charlie_account),
        U512::from(10000000000000u64),
    );

    context.deposit_into(
        context.bob_account,
        Key::Account(context.charlie_account),
        U512::from(10000000000000u64),
    );

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.1, U512::from(38500000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // charlie collects the deposits
    context.collect(context.charlie_account);

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.1, U512::from(38500000000000_u64));
    assert_eq!(account_balances.2, U512::from(68500000000000_u64));
}
