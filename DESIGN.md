# Multitest Design

Multitest is designed to simulate a blockchain environment in pure Rust, specifically for unit testing scenarios involving contract-to-contract and contract-to-bank interactions. This framework is a simulation of the Cosmos SDK x/wasm module, ideal for testing multi-contract deployments before live blockchain deployment.

This explains some of the design for those who want to use the API, as well as those who want to look under the hood.

## Key APIs

### App

The 'App' serves as the main entry point to the 'cw-multi-test' system, representing a simulated blockchain application. 

It manages the concept of block height and time, which are essential for simulating the progression of blocks in a blockchain environment. The 'App' allows for the execution of 'CosmosMsg' as atomic transactions, meaning the state changes only commit if the execution is successful. Additionally, it provides a querying interface for interacting with the blockchain state.

It exposes an entry point `App.execute` that allows us to execute any `CosmosMsg`
and it wraps it as an atomic transaction. That is, only if `execute` returns success, will the state
be committed. It returns the data and a list of Events on successful execution or an `Err(String)`
on error. There are some helper methods tied to the `Executor` trait that create the `CosmosMsg` for
you to provide a less verbose API. `instantiate_contract`,`execute_contract`, and `send_tokens` are exposed
for your convenience in writing tests. Each execute one `CosmosMsg` atomically as if it was submitted by a user.
(You can also use `execute_multi` if you wish to run multiple message together that revert all state if any fail).

The other key entry point to `App` is the `Querier` interface that it implements. In particular, you
can use `App.wrap()` to get a `QuerierWrapper`, which provides all kinds of nice APIs to query the
blockchain, like `all_balances` and `query_wasm_smart`. Putting this together, you have one `Storage` wrapped
into an application, where you can execute contracts and bank, query them easily, and update the current
`BlockInfo`, in an API that is not very verbose or cumbersome. Under the hood it will process all messages
returned from contracts, move "bank" tokens and call into other contracts.

You can create an App for use in your testcode like:

```rust
fn mock_app() -> App {
    let env = mock_env();
    let api = Box::new(MockApi::default());
    let bank = BankKeeper::new();

    App::new(api, env.block, bank, Box::new(MockStorage::new()))
}
```

Inside App, it maintains the root `Storage`, and the `BlockInfo` for the current block.
It also contains a `Router` (discussed below), which can process any `CosmosMsg` variant
by passing it to the proper "Keeper".

Note: This properly handles submessages and reply blocks.

Note: While the API currently supports custom messages, we don't currently have a way to handle/process them.

### Contracts

Before you can call contracts in the `cw-multi-test` environment, they must first be instantiated using a `code_id`. In a live blockchain like `wasmd`, `code_id` refers to stored Wasm code. In `cw-multi-test`, it instead points to a `Box<dyn Contract>` that will be executed.

The `Contract` trait is central to this process, defining the major entry points to any CosmWasm contract, including `execute`, `instantiate`, `query`, `sudo`, and `reply` (for submessages). These functions are crucial for the contract's interaction within the simulated blockchain environment.

In order to easily implement `Contract` from some existing contract code, we use the `ContractWrapper` struct,
which takes some function pointers and combines them. You can look in `test_helpers.rs` for some examples
or how to do so (and useful mocks for some test cases). Here is an example of wrapping a CosmWasm contract into
a `Contract` trait to add to an `App`:

```rust
use cw20_escrow::contract::{ execute, instantiate, query };

pub fn contract_escrow() -> Box<dyn Contract<Empty>> {
  let contract = ContractWrapper::new(execute, instantiate, query);
  Box::new(contract)
}
```

If you are not using custom messages in your contract, you can just use `dyn Contract<Empty>`.

### Examples

The best intro is most likely `integration.rs` in `cw20-escrow`, which shows sending and releasing native tokens in
an escrow, as well as sending and releasing cw20 tokens. The first one updates the global bank ledger, the second
actually shows how we can test orchestrating multiple contracts.

## Implementation

Besides the `App` and `Contract` interfaces which are the primary means with interacting with this module,
there are a number of components that need to be understood if you wish to extend the module (say, adding
a MockStaking module to handle `CosmosMsg::Staking` and `QueryRequest::Staking` calls).

### StorageTransaction

Since much of the logic, both on the app side, as well as in submessages, relies on rolling back any changes
if there is an error, we make heavy use of `StorageTransaction` under the hood. It takes a `&Storage`
reference and produces `&mut Storage` that can be written too. Notably, we can still query the original
(snapshot) storage while writing (which is very useful for the `Querier` interface for contracts).

You can drop the `StorageTransaction` causing the changes to be rolled back (well, never committed),
or on success, you can commit it to the underlying storage. Note that there may be multiple levels
or `StorageTransaction` wrappers above the root (App) storage. Here is an example of using it,
that should make the concepts clear:

```rust
// execute in cache
let mut cache = StorageTransaction::new(storage);
// Note that we *could* query the original `storage` while `cache` is live
let res = router.execute(&mut cache, block, contract.clone(), msg.msg);
if res.is_ok() {
    cache.prepare().commit(storage);
}
```

### Modules

Modules in 'cw-multi-test' are akin to the Cosmos SDK, where each module handles specific message variants. For instance, a 'Bank' module might handle 'BankMsg' and 'BankQuery'. Each module provides execute and query support, as well as "admin" methods that are necessary for setting up test cases but aren't exposed to end users.

Each module produces a soon-to-be standardized interface to interact with. It exposes `execute` and `query` support
as well as some "admin" methods that cannot be called by users but are needed for testcase setup. I am working on a
design to make these "admin" methods more extensible as well. If you look at the two existing modules, you can
see the great similarity in `query` and `execute`, such that we could consider making a `Module<MSG, QUERY>` trait.

```rust
pub trait Wasm<C>
where
    C: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Handles all WasmQuery requests
    fn query(
        &self,
        storage: &dyn Storage,
        querier: &dyn Querier,
        block: &BlockInfo,
        request: WasmQuery,
    ) -> Result<Binary, String>;

    /// Handles all WasmMsg messages
    fn execute(
        &self,
        storage: &mut dyn Storage,
        router: &Router<C>,
        block: &BlockInfo,
        sender: Addr,
        msg: WasmMsg,
    ) -> Result<AppResponse, String>;

    // Add a new contract. Must be done on the base object, when no contracts running
    fn store_code(&mut self, code: Box<dyn Contract<C>>) -> usize;

    /// Admin interface, cannot be called via CosmosMsg
    fn sudo(
        &self,
        contract_addr: Addr,
        storage: &mut dyn Storage,
        router: &Router<C>,
        block: &BlockInfo,
        msg: Vec<u8>,
    ) -> Result<AppResponse, String>;
}
```

```rust
/// Bank is a minimal contract-like interface that implements a bank module
/// It is initialized outside of the trait
pub trait Bank {
    fn execute(
        &self,
        storage: &mut dyn Storage,
        sender: Addr,
        msg: BankMsg,
    ) -> Result<AppResponse, String>;

    fn query(&self, storage: &dyn Storage, request: BankQuery) -> Result<Binary, String>;

    // Admin interface
    fn init_balance(
        &self,
        storage: &mut dyn Storage,
        account: &Addr,
        amount: Vec<Coin>,
    ) -> Result<(), String>;
}
```

These traits should capture all public interactions with the module ("Keeper interface" if you come from
Cosmos SDK terminology). All other methods on the implementations should be private (or at least not exposed
outside of the multitest crate).

### Router

The Router in 'cw-multi-test' is a critical component that integrates all individual modules into a cohesive whole, capable of processing any 'CosmosMsg'. For example, it routes 'WasmMsg' to the Wasm module and 'BankMsg' to the Bank module. The Router ensures that messages are dispatched to the appropriate module for processing, facilitating complex interactions within the simulated environment.

```rust
impl<C> Router<C> {
  pub fn execute(
    &self,
    storage: &mut dyn Storage,
    block: &BlockInfo,
    sender: Addr,
    msg: CosmosMsg<C>,
  ) -> Result<AppResponse, String> {
    match msg {
      CosmosMsg::Wasm(msg) => self.wasm.execute(storage, &self, block, sender, msg),
      // FIXME: we could pass in unused router and block for consistency
      CosmosMsg::Bank(msg) => self.bank.execute(storage, sender, msg),
      _ => unimplemented!(),
    }
  }
}
```

Note that the only way one module can call or query another module is by dispatching messages via the `Router`.
This allows us to implement an independent `Wasm` in a way that it can process `SubMsg` that call into `Bank`.
You can see an example of that in WasmKeeper.send, where it moves bank tokens from one account to another:

```rust
impl WasmKeeper {
  fn send<T: Into<Addr>>(
    &self,
    storage: &mut dyn Storage,
    router: &Router<C>,
    block: &BlockInfo,
    sender: T,
    recipient: String,
    amount: &[Coin],
  ) -> Result<AppResponse, String> {
    if !amount.is_empty() {
      let msg = BankMsg::Send {
        to_address: recipient,
        amount: amount.to_vec(),
      };
      let res = router.execute(storage, block, sender.into(), msg.into())?;
      Ok(res)
    } else {
      Ok(AppResponse::default())
    }
  }
}
```
