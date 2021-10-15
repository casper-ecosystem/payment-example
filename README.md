
# TODO rework


# Payment Contract Example

This example demonstrates the effective methods of transferring motes. Motes are contained inside purses.
Each account on the network has its own default purse, creation of new purses inside contracts is possible.

## Contract Entrypoints
- "deposit"
    - parameters:
        - "purse": CLType::URef
        - return: CLType::Unit

- "collect",
    - parameters:
        - "recipient": AccountHash::cl_type())
    - return: CLType::Unit,
    - access groups: "group_label"

## Test accounts data
- admin
    - private key: `[1u8;32]`
    - account addr: `ef4687f74d465826239bab05c4e1bdd2223dd8c201b96f361f775125e624ef70`
    - purse_addr: `486e322928be0239e1ee99888cdca5be4e84cbce32b276903718c63fa84cc392`

- participant_dos
    - private key: `[2u8;32]`
    - account addr: `a6f341ee3d5124163c75a93364df7556b1763313554d3abf4bedc8206d94c1b2`
    - purse_addr: `d124a145ff53378cfff7970ca163b2acee77570c6c6852fc717de5067c325db6`

- participant_tres
    - private key: `[3u8;32]`
    - account addr: `67e7554760e6a57150ca567bdf38cc46ed178b5e688842ede7b854e8eabe5d80`
    - purse_addr: `3742e6011967754e97d28b35be8d915159db7b05370f802e79c9d4507f075e04`
    
## test snippets:
- to get balance
```
    let purse_addr : [u8;32] = hex::decode("486e322928be0239e1ee99888cdca5be4e84cbce32b276903718c63fa84cc392").unwrap().as_slice().try_into().unwrap();
    context.get_balance(purse_addr));
```

- to get account (has purse_addr in it)
```
    context.get_account(tres_account_addr)
```

## contract snippets:
- transfer from caller accounts purse to a purse that is the main purse of an outside account, from inside contract:
```
    let recipient: AccountHash = runtime::get_named_arg("recipient");
    let amount: U512 = runtime::get_named_arg("amount");
    let main_purse = account::get_main_purse();
    system::transfer_from_purse_to_account(main_purse, recipient, amount, None)
        .unwrap_or_revert();
```
or shorter version for same purpose (skips manually getting the callers purse):
```
    let recipient : AccountHash = runtime::get_named_arg("recipient");
    let amount : U512 = runtime::get_named_arg("amount");
    system::transfer_to_account(recipient, amount, None)
        .unwrap_or_revert();
```

- transfer from account to another purse inside contract:
```
    let main_purse = account::get_main_purse();
    let amount: U512 = runtime::get_named_arg("amount");
    let local_purse = system::create_purse();
    system::transfer_from_purse_to_purse(main_purse, local_purse, amount, None)
        .unwrap_or_revert();
```

- get purse the balance of any purse that gets passed in as parameter:
```
    let main_purse = account::get_main_purse();
    system::get_purse_balance(main_purse); 
```
or the following for short to get caller account balance:
```
    system::get_balance();
```

- create purse for in-contract use:
```
    system::create_purse()
```

## Notes

One thing is important to mention that not every purse action is available from anywhere.
`transfer_from_purse_to_purse` is only possible when you caller have access rights to both purses.
For example the main purse of an account. To and from this purse only session level contract contexts can make transfers.
If you want to `transfer_from_purse_to_account` on the other hand, you need only have access right to one purse your transfer from.

### Versions
This example is on casper-types and casper-contract version 1.3.4
rustc 1.56.0-nightly (2faabf579 2021-07-27)