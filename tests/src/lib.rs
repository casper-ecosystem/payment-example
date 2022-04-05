use std::path::PathBuf;

use casper_engine_test_support::{InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST};

use casper_types::{account::AccountHash, runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};
use casper_types::{ContractHash, ContractPackageHash, Key};
use utils::{deploy, fund_account, query, DeploySource};

mod utils;

pub struct PaymentContract {
    pub builder: InMemoryWasmTestBuilder,
    pub contract_hash: ContractHash,
    pub package_hash: ContractPackageHash,
    pub admin_account: (PublicKey, AccountHash),
    pub participant_two: (PublicKey, AccountHash),
    pub participant_three: (PublicKey, AccountHash),
}

impl PaymentContract {
    pub fn deploy() -> Self {
        // We create 3 users. One to oversee and deploy the contract, one to send the payment
        // and one to receive it.
        let admin_public_key: PublicKey =
            PublicKey::from(&SecretKey::ed25519_from_bytes([1u8; 32]).unwrap());
        let participant_two_public_key: PublicKey =
            PublicKey::from(&SecretKey::ed25519_from_bytes([2u8; 32]).unwrap());
        let participant_three_public_key: PublicKey =
            PublicKey::from(&SecretKey::ed25519_from_bytes([3u8; 32]).unwrap());
        // Get addresses for participating users.
        let admin_account_addr = AccountHash::from(&admin_public_key);
        let participant_two_account_addr = AccountHash::from(&participant_two_public_key);
        let participant_three_account_addr = AccountHash::from(&participant_three_public_key);

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder
            .exec(fund_account(&admin_account_addr))
            .expect_success()
            .commit();
        builder
            .exec(fund_account(&participant_two_account_addr))
            .expect_success()
            .commit();
        builder
            .exec(fund_account(&participant_three_account_addr))
            .expect_success()
            .commit();

        // load contract into context
        let code = PathBuf::from("escrow.wasm");
        deploy(
            &mut builder,
            &admin_account_addr,
            &DeploySource::Code(code),
            runtime_args! {},
            true,
            None,
        );

        let contract_hash = query(
            &builder,
            Key::Account(admin_account_addr),
            &["escrow_contract_hash".to_string()],
        );

        let package_hash: ContractPackageHash = query(
            &builder,
            Key::Account(admin_account_addr),
            &[
                "escrow_contract".to_string(),
                "escrow_contract_package".to_string(),
            ],
        );

        Self {
            builder,
            contract_hash,
            package_hash,
            admin_account: (admin_public_key, admin_account_addr),
            participant_two: (participant_two_public_key, participant_two_account_addr),
            participant_three: (participant_three_public_key, participant_three_account_addr),
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
            self.get_balance(&self.admin_account.1),
            self.get_balance(&self.participant_two.1),
            self.get_balance(&self.participant_three.1),
        )
    }

    /// Function that handles the creation and running of sessions.
    fn call(&mut self, caller: AccountHash, method: &str, args: RuntimeArgs) {
        // let code = Code::Hash(self.contract_hash, method.to_string());
        // let session = SessionBuilder::new(code, args)
        //     .with_address(caller)
        //     .with_authorization_keys(&[caller])
        //     .build();
        // self.context.run(session);
        deploy(
            &mut self.builder,
            &caller,
            &DeploySource::ByPackageHash {
                package_hash: self.package_hash,
                method: method.to_string(),
            },
            args,
            true,
            None,
        );
    }

    /// Calls the additional "deposit" contract with Key::Account(recipient) and the hash of the escrow contract,
    /// that creates a purse and transfers 100000000000000000 motes into it,
    /// then transfers said purse to the escrow contract.
    pub fn deposit(&mut self, sender: AccountHash, recipient: Key, amount: U512) {
        let code = PathBuf::from("deposit.wasm");
        let args = runtime_args! {
            "escrow_contract_package" => self.package_hash,
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

    /// Function that calls the `collect` endpoint on the escrow contract,
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

    // Print the balance of all 3 users

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(48500000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // send tokens from admin to contract
    context.deposit(
        context.admin_account.1,
        Key::Account(context.participant_three.1),
        U512::from(10000000000000u64),
    );

    // look at balances again, admins money should be down by a deposited 10000000000000,
    // and another 1500000000000 that is the contract deployment cost in the tests.
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // collect token to a third account
    context.collect(context.participant_three.1);

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

    // Admin and Participant Two deposits money for Participant Three
    context.deposit(
        context.admin_account.1,
        Key::Account(context.participant_three.1),
        U512::from(10000000000000u64),
    );

    context.deposit(
        context.participant_two.1,
        Key::Account(context.participant_three.1),
        U512::from(10000000000000u64),
    );

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.1, U512::from(38500000000000_u64));
    assert_eq!(account_balances.2, U512::from(50000000000000_u64));

    // Participant Three collects their money
    context.collect(context.participant_three.1);

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(37000000000000_u64));
    assert_eq!(account_balances.1, U512::from(38500000000000_u64));
    assert_eq!(account_balances.2, U512::from(68500000000000_u64));
}
