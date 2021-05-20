#[cfg(test)]
mod tests {
    use casper_engine_test_support::{Code, Hash, SessionBuilder, TestContext, TestContextBuilder};
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, PublicKey, RuntimeArgs,
        SecretKey, URef, U512,
    };
    use std::convert::TryInto;

    pub struct PaymentContract {
        pub context: TestContext,
        pub contract_hash: Hash,
        pub admin_account: (PublicKey, AccountHash),
        pub participant_two: (PublicKey, AccountHash),
        pub participant_three: (PublicKey, AccountHash),
    }

    impl PaymentContract {
        pub fn deploy() -> Self {
            // We create 3 users. One to oversee and deploy the contract, one to send the payment
            // and one to receive it.
            let admin_public_key: PublicKey = SecretKey::ed25519([1u8; 32]).into();
            let participant_two_public_key: PublicKey = SecretKey::ed25519([2u8; 32]).into();
            let participant_three_public_key: PublicKey = SecretKey::ed25519([3u8; 32]).into();
            // Get addresses for participating users.
            let admin_account_addr = AccountHash::from(&admin_public_key);
            let participant_two_account_addr = AccountHash::from(&participant_two_public_key);
            let participant_three_account_addr = AccountHash::from(&participant_three_public_key);

            // create context with cash for all users
            let clx_init_balance = U512::from(500_000_000_000_000_000u64);
            let mut context = TestContextBuilder::new()
                .with_public_key(admin_public_key, clx_init_balance)
                .with_public_key(participant_two_public_key, clx_init_balance)
                .with_public_key(participant_three_public_key, clx_init_balance)
                .build();

            // load contract into context
            let code = Code::from("payment_contract.wasm");
            let args = runtime_args! {};
            let session = SessionBuilder::new(code, args)
                .with_address(admin_account_addr)
                .with_authorization_keys(&[admin_account_addr])
                .build();
            context.run(session);

            let contract_hash = context
                .query(admin_account_addr, &["payment_contract_hash".to_string()])
                .unwrap_or_else(|_| panic!("payment_contract_hash contract not found"))
                .into_t()
                .unwrap_or_else(|_| panic!("payment_contract_hash has wrong type"));

            Self {
                context,
                contract_hash,
                admin_account: (admin_public_key, admin_account_addr),
                participant_two: (participant_two_public_key, participant_two_account_addr),
                participant_three: (participant_three_public_key, participant_three_account_addr),
            }
        }

        fn hash_bytes_from_hex_str(hex: &str) -> [u8; 32] {
            hex::decode(hex).unwrap().as_slice().try_into().unwrap()
        }

        pub fn get_all_accounts_balance(&self) -> (U512, U512, U512) {
            (
                self.context.get_balance(Self::hash_bytes_from_hex_str(
                    "486e322928be0239e1ee99888cdca5be4e84cbce32b276903718c63fa84cc392",
                )),
                self.context.get_balance(Self::hash_bytes_from_hex_str(
                    "d124a145ff53378cfff7970ca163b2acee77570c6c6852fc717de5067c325db6",
                )),
                self.context.get_balance(Self::hash_bytes_from_hex_str(
                    "3742e6011967754e97d28b35be8d915159db7b05370f802e79c9d4507f075e04",
                )),
            )
        }

        fn _query_contract<T: CLTyped + FromBytes>(
            &self,
            caller: AccountHash,
            name: &str,
        ) -> Option<T> {
            match self.context.query(
                caller,
                &["payment_contract_hash".to_string(), name.to_string()],
            ) {
                Err(e) => {
                    println!("query_contract target {} resulted in error: {:?}", name, e);
                    None
                }
                Ok(maybe_value) => {
                    let value = maybe_value
                        .into_t()
                        .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                    Some(value)
                }
            }
        }

        fn _call(&mut self, caller: AccountHash, method: &str, args: RuntimeArgs) {
            let code = Code::Hash(self.contract_hash, method.to_string());
            let session = SessionBuilder::new(code, args)
                .with_address(caller)
                .with_authorization_keys(&[caller])
                .build();
            self.context.run(session);
        }

        pub fn send_tokens(&mut self, sender: AccountHash) {
            let code = Code::from("send_tokens.wasm");
            let args = runtime_args! {
                "payment_contract" => self.contract_hash
            };
            let session = SessionBuilder::new(code, args)
                .with_address(sender)
                .with_authorization_keys(&[sender])
                .build();
            self.context.run(session);
        }

        pub fn collect(&mut self, sender: AccountHash, recipient: AccountHash) {
            let code = Code::from("collect.wasm");
            // if we ask for as the amount than there is in the contract, we only collect what's in the contract.
            let args = runtime_args! {
                "payment_contract" => self.contract_hash,
                "recipient" => recipient,
                "amount" => U512::from(100000000000000000u64)
            };
            let session = SessionBuilder::new(code, args)
                .with_address(sender)
                .with_authorization_keys(&[sender])
                .build();
            self.context.run(session);
        }

        pub fn get_contract_balance(&self) -> U512 {
            let contract_purse: URef = self
                .context
                .query(
                    self.admin_account.1,
                    &[
                        "payment_contract".to_string(),
                        "contract_purse_wrapper".to_string(),
                    ],
                )
                .unwrap()
                .into_t()
                .unwrap();
            let balance = self.context.get_balance(contract_purse.addr());
            balance
        }
    }

    #[test]
    fn test_payment() {
        // Setup example contract context
        let mut context = PaymentContract::deploy();

        // Print the balance of all 3 users
        println!("1: Accounts: {:?}", context.get_all_accounts_balance());
        println!("1: Contract: {:?}", context.get_contract_balance());

        // send tokens from admin to contract
        context.send_tokens(context.admin_account.1);

        // look at balances again
        println!("2: Accounts: {:?}", context.get_all_accounts_balance());
        println!("2: Contract: {:?}", context.get_contract_balance());

        // collect token to a third account
        context.collect(context.admin_account.1, context.participant_three.1);

        // tokens are retrieved
        println!("3: Accounts: {:?}", context.get_all_accounts_balance());
        println!("3: Contract: {:?}", context.get_contract_balance());

        // another user tries to send tokens to the contract
        context.send_tokens(context.participant_three.1);
        println!("4: Accounts: {:?}", context.get_all_accounts_balance());
        println!("4: Contract: {:?}", context.get_contract_balance());

        // this next should fail, because participant_two does not have the authority to collect tokens
        context.collect(context.participant_two.1, context.participant_two.1);

        println!("5: Accounts: {:?}", context.get_all_accounts_balance());
        println!("5: Contract: {:?}", context.get_contract_balance());
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
