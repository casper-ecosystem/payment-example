use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
use casper_types::{account::AccountHash, runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};
use casper_types::{ContractPackageHash, Key};

pub struct PaymentContract {
    pub context: TestContext,
    pub contract_hash: Hash,
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
            (&SecretKey::ed25519_from_bytes([1u8; 32]).unwrap()).into();
        let participant_two_public_key: PublicKey =
            (&SecretKey::ed25519_from_bytes([2u8; 32]).unwrap()).into();
        let participant_three_public_key: PublicKey =
            (&SecretKey::ed25519_from_bytes([3u8; 32]).unwrap()).into();
        // Get addresses for participating users.
        let admin_account_addr = AccountHash::from(&admin_public_key);
        let participant_two_account_addr = AccountHash::from(&participant_two_public_key);
        let participant_three_account_addr = AccountHash::from(&participant_three_public_key);

        // create context with cash for all users
        let clx_init_balance = U512::from(500_000_000_000_000_000u64);
        let mut context = TestContextBuilder::new()
            .with_public_key(admin_public_key.clone(), clx_init_balance)
            .with_public_key(participant_two_public_key.clone(), clx_init_balance)
            .with_public_key(participant_three_public_key.clone(), clx_init_balance)
            .build();

        // load contract into context
        let code = Code::from("escrow.wasm");
        let args = runtime_args! {};
        let session = SessionBuilder::new(code, args)
            .with_address(admin_account_addr)
            .with_authorization_keys(&[admin_account_addr])
            .build();
        context.run(session);

        let contract_hash = context
            .query(admin_account_addr, &["escrow_contract_hash".to_string()])
            .unwrap_or_else(|_| panic!("escrow_contract_hash contract not found"))
            .into_t()
            .unwrap_or_else(|_| panic!("escrow_contract_hash has wrong type"));

        let package_hash: ContractPackageHash = context
            .query(
                admin_account_addr,
                &[
                    "escrow_contract".to_string(),
                    "escrow_contract_package".to_string(),
                ],
            )
            .unwrap()
            .into_t()
            .unwrap();

        Self {
            context,
            contract_hash,
            package_hash,
            admin_account: (admin_public_key, admin_account_addr),
            participant_two: (participant_two_public_key, participant_two_account_addr),
            participant_three: (participant_three_public_key, participant_three_account_addr),
        }
    }
    /// Getter function for the balance of an account.
    fn get_balance(&self, account_key: &AccountHash) -> U512 {
        let main_purse_address = self.context.main_purse_address(*account_key).unwrap();
        self.context.get_balance(main_purse_address.addr())
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
        let code = Code::Hash(self.contract_hash, method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(caller)
            .with_authorization_keys(&[caller])
            .build();
        self.context.run(session);
    }

    /// Calls the additional "deposit" contract with Key::Account(recipient) and the hash of the escrow contract,
    /// that creates a purse and transfers 100000000000000000 motes into it,
    /// then transfers said purse to the escrow contract.
    pub fn deposit(&mut self, sender: AccountHash, recipient: Key) {
        let code = Code::from("deposit.wasm");
        let args = runtime_args! {
            "escrow_contract_package" => self.package_hash,
            "recipient" => recipient,
        };
        let session = SessionBuilder::new(code, args)
            .with_address(sender)
            .with_authorization_keys(&[sender])
            .build();
        self.context.run(session);
    }

    /// Function that calls the `collect` endpoint on the escrow contract,
    /// that directly transfers the amount in the purse stored to the accounts hash to the account.
    pub fn collect(&mut self, recipient: AccountHash) {
        self.call(recipient, "collect", runtime_args! {});
    }
}

#[test]
fn test_payment_and_collect() {
    // Setup example contract context
    let mut context = PaymentContract::deploy();

    // Print the balance of all 3 users

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(499998500000000000_u64));
    assert_eq!(account_balances.2, U512::from(500000000000000000_u64));

    // send tokens from admin to contract
    context.deposit(
        context.admin_account.1,
        Key::Account(context.participant_three.1),
    );

    // look at balances again, admins money should be down by a deposited 100000000000000000,
    // and another 1500000000000 that is the contract deployment cost in the tests.
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(399997000000000000_u64));
    assert_eq!(account_balances.2, U512::from(500000000000000000_u64));

    // collect token to a third account
    context.collect(context.participant_three.1);

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(399997000000000000_u64));
    assert_eq!(account_balances.2, U512::from(599998500000000000_u64));
}

#[test]
fn test_multiple_payment_and_single_collect() {
    // Setup example contract context
    let mut context = PaymentContract::deploy();

    // Default state (after deployment)
    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(499998500000000000_u64));
    assert_eq!(account_balances.1, U512::from(500000000000000000_u64));
    assert_eq!(account_balances.2, U512::from(500000000000000000_u64));

    // Admin and Participant Two deposits money for Participant Three
    context.deposit(
        context.admin_account.1,
        Key::Account(context.participant_three.1),
    );

    context.deposit(
        context.participant_two.1,
        Key::Account(context.participant_three.1),
    );

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(399997000000000000_u64));
    assert_eq!(account_balances.1, U512::from(399998500000000000_u64));
    assert_eq!(account_balances.2, U512::from(500000000000000000_u64));

    // Participant Three collects their money
    context.collect(context.participant_three.1);

    let account_balances = context.get_all_accounts_balance();
    assert_eq!(account_balances.0, U512::from(399997000000000000_u64));
    assert_eq!(account_balances.1, U512::from(399998500000000000_u64));
    assert_eq!(account_balances.2, U512::from(699998500000000000_u64));
}
