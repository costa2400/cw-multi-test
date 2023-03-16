//! Traits implemented for anything which can be an entry point, so it is aligned to inner
//! multi-test structure. They are defined for any function-like object with matching signatures,
//! being extension traits for `Fn`

use std::marker::PhantomData;

use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{Binary, CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response};
use serde::Deserialize;

use super::context::{CustomizeDepsMut, CustomizeResponse};

/// `execute` or `instantiate` entry point
///
/// * `Q` - a blockchain-specific query-type
/// * `C` - a blockchain-specific custom-type
pub trait ContractFn<Q, C>
where
    Q: CustomQuery,
{
    /// A message the entry point is handling
    type Msg: for<'de> Deserialize<'de>;

    fn call(
        &self,
        deps: DepsMut<Q>,
        env: Env,
        info: MessageInfo,
        msg: Self::Msg,
    ) -> AnyResult<Response<C>>;
}

// It would be preferable for those functions to be provided `ContractFn` trait, but it is impossible due
// to need of returning `impl ContractFn`.
fn cast_query<NewQ, F, Q, C>(f: F) -> impl ContractFn<NewQ, C>
where
    F: ContractFn<Q, C>,
    NewQ: CustomQuery,
    Q: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
{
    (move |deps: DepsMut<NewQ>, env: Env, info: MessageInfo, msg: F::Msg| {
        f.call(deps.customize(), env, info, msg)
    })
    .wrap()
}

fn cast_msg<NewC, F, Q, C>(f: F) -> impl ContractFn<Q, NewC>
where
    F: ContractFn<Q, C>,
    Q: CustomQuery,
    Response<C>: CustomizeResponse<NewC>,
{
    (move |deps: DepsMut<Q>, env: Env, info: MessageInfo, msg: F::Msg| {
        f.call(deps, env, info, msg).map(|resp| resp.customize())
    })
    .wrap()
}

fn cast_fn<NewQ, NewC, F, Q, C>(f: F) -> impl ContractFn<NewQ, NewC>
where
    F: ContractFn<Q, C>,
    Q: CustomQuery,
    NewQ: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
    Response<C>: CustomizeResponse<NewC>,
{
    (move |deps: DepsMut<NewQ>, env: Env, info: MessageInfo, msg: F::Msg| {
        f.call(deps.customize(), env, info, msg)
            .map(|resp| resp.customize())
    })
    .wrap()
}

impl<T, Q, C, E> ContractFn<Q, C> for fn(DepsMut<Q>, Env, MessageInfo, T) -> Result<Response<C>, E>
where
    T: for<'de> Deserialize<'de>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    type Msg = T;

    fn call(
        &self,
        deps: DepsMut<Q>,
        env: Env,
        info: MessageInfo,
        msg: T,
    ) -> AnyResult<Response<C>> {
        self(deps, env, info, msg).map_err(Into::into)
    }
}

/// `sudo` or `migrate` entry point
///
/// * `Q` - a blockchain-specific query-type
/// * `C` - a blockchain-specific custom-type
pub trait PermissionedFn<Q, C>
where
    Q: CustomQuery,
{
    /// A message the entry point is handling
    type Msg: for<'de> Deserialize<'de>;

    fn call(&self, deps: DepsMut<Q>, env: Env, msg: Self::Msg) -> AnyResult<Response<C>>;
}

// It would be preferable for those functions to be provided `PermissionedFn` trait, but it is impossible due
// to need of returning `impl PermissionedFn`.
fn cast_permissioned_query<NewQ, F, Q, C>(f: F) -> impl PermissionedFn<NewQ, C>
where
    F: PermissionedFn<Q, C>,
    NewQ: CustomQuery,
    Q: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
{
    (move |deps: DepsMut<NewQ>, env: Env, msg: F::Msg| f.call(deps.customize(), env, msg)).wrap()
}

fn cast_permissioned_msg<NewC, F, Q, C>(f: F) -> impl PermissionedFn<Q, NewC>
where
    F: PermissionedFn<Q, C>,
    Q: CustomQuery,
    Response<C>: CustomizeResponse<NewC>,
{
    (move |deps: DepsMut<Q>, env: Env, msg: F::Msg| {
        f.call(deps, env, msg).map(|resp| resp.customize())
    })
    .wrap()
}

fn cast_permissioned_fn<NewQ, NewC, F, Q, C>(f: F) -> impl PermissionedFn<NewQ, NewC>
where
    F: PermissionedFn<Q, C>,
    Q: CustomQuery,
    NewQ: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
    Response<C>: CustomizeResponse<NewC>,
{
    (move |deps: DepsMut<NewQ>, env: Env, msg: F::Msg| {
        f.call(deps.customize(), env, msg)
            .map(|resp| resp.customize())
    })
    .wrap()
}

impl<T, Q, C, E> PermissionedFn<Q, C> for fn(DepsMut<Q>, Env, T) -> Result<Response<C>, E>
where
    T: for<'de> Deserialize<'de>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    type Msg = T;

    fn call(&self, deps: DepsMut<Q>, env: Env, msg: T) -> AnyResult<Response<C>> {
        self(deps, env, msg).map_err(Into::into)
    }
}

/// `reply` entry point
///
/// * `Q` - a blockchain-specific query-type
/// * `C` - a blockchain-specific custom-type
pub trait ReplyFn<Q, C>
where
    Q: CustomQuery,
{
    fn call(&self, deps: DepsMut<Q>, env: Env, msg: Reply) -> AnyResult<Response<C>>;
}

impl<F, Q, C, E> ReplyFn<Q, C> for F
where
    F: Fn(DepsMut<Q>, Env, Reply) -> Result<Response<C>, E>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    fn call(&self, deps: DepsMut<Q>, env: Env, msg: Reply) -> AnyResult<Response<C>> {
        self(deps, env, msg).map_err(Into::into)
    }
}

// It would be preferable for those functions to be provided `ReplyFn` trait, but it is impossible due
// to need of returning `impl ReplyFn`.
fn cast_reply_query<NewQ, F, Q, C>(f: F) -> impl ReplyFn<NewQ, C>
where
    F: ReplyFn<Q, C>,
    NewQ: CustomQuery,
    Q: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
{
    move |deps: DepsMut<NewQ>, env: Env, msg: Reply| f.call(deps.customize(), env, msg)
}

fn cast_reply_msg<NewC, F, Q, C>(f: F) -> impl ReplyFn<Q, NewC>
where
    F: ReplyFn<Q, C>,
    Q: CustomQuery,
    Response<C>: CustomizeResponse<NewC>,
{
    move |deps: DepsMut<Q>, env: Env, msg: Reply| {
        f.call(deps, env, msg).map(|resp| resp.customize())
    }
}

fn cast_reply_fn<NewQ, NewC, F, Q, C>(f: F) -> impl ReplyFn<NewQ, NewC>
where
    F: ReplyFn<Q, C>,
    Q: CustomQuery,
    NewQ: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
    Response<C>: CustomizeResponse<NewC>,
{
    move |deps: DepsMut<NewQ>, env: Env, msg: Reply| {
        f.call(deps.customize(), env, msg)
            .map(|resp| resp.customize())
    }
}

/// `query` entry point
///
/// * `Q` - a blockchain-specific query-type
pub trait QueryFn<Q>
where
    Q: CustomQuery,
{
    type Msg: for<'de> Deserialize<'de>;

    fn call(&self, deps: Deps<Q>, env: Env, msg: Self::Msg) -> AnyResult<Binary>;
}

// It would be preferable for those functions to be provided `ReplyFn` trait, but it is impossible due
// to need of returning `impl ReplyFn`.
fn cast_query<NewQ, F, Q>(f: F) -> impl QueryFn<NewQ>
where
    F: QueryFn<Q>,
    NewQ: CustomQuery,
    Q: CustomQuery,
    for<'a> DepsMut<'a, NewQ>: CustomizeDepsMut<'a, Q>,
{
    (move |deps: DepsMut<NewQ>, env: Env, msg: F::Msg| f.call(deps.customize(), env, msg)).wrap()
}

impl<T, Q, E> QueryFn<Q> for fn(Deps<Q>, Env, T) -> Result<Binary, E>
where
    T: for<'de> Deserialize<'de>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    type Msg = T;

    fn call(&self, deps: Deps<Q>, env: Env, msg: T) -> AnyResult<Binary> {
        self(deps, env, msg).map_err(Into::into)
    }
}

/// Thin wrapper for types implementing proper `Fn` traits, but not being native functions.
///
/// This is required, as for every type it is possible that on the single type, multiple `Fn`
/// traits might be implemented, so the `Msg` has to be fixed.
pub struct FnWrapper<F, Msg> {
    f: F,
    _phantom: PhantomData<Msg>,
}

impl<F, T, Q, C, E> ContractFn<Q, C> for FnWrapper<F, T>
where
    F: Fn(DepsMut<Q>, Env, MessageInfo, T) -> Result<Response<C>, E>,
    T: for<'de> Deserialize<'de>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    type Msg = T;

    fn call(
        &self,
        deps: DepsMut<Q>,
        env: Env,
        info: MessageInfo,
        msg: T,
    ) -> AnyResult<Response<C>> {
        (self.f)(deps, env, info, msg).map_err(Into::into)
    }
}

impl<F, T, Q, C, E> PermissionedFn<Q, C> for FnWrapper<F, T>
where
    F: Fn(DepsMut<Q>, Env, T) -> Result<Response<C>, E>,
    T: for<'de> Deserialize<'de>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    type Msg = T;

    fn call(&self, deps: DepsMut<Q>, env: Env, msg: T) -> AnyResult<Response<C>> {
        (self.f)(deps, env, msg).map_err(Into::into)
    }
}

impl<F, T, Q, E> QueryFn<Q> for FnWrapper<F, T>
where
    F: Fn(Deps<Q>, Env, T) -> Result<Binary, E>,
    T: for<'de> Deserialize<'de>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    type Msg = T;

    fn call(&self, deps: Deps<Q>, env: Env, msg: T) -> AnyResult<Binary> {
        (self.f)(deps, env, msg).map_err(Into::into)
    }
}

/// Utility trait to make `Fn` implementing types into `FnWrapper` so they are useable as entry
/// points for `ContractWrapper`.
pub trait WrappableFn<Msg>
where
    Self: Sized,
{
    fn wrap(self) -> FnWrapper<Self, Msg> {
        FnWrapper {
            f: self,
            _phantom: PhantomData,
        }
    }
}

impl<T, Msg> WrappableFn<Msg> for T {}

/// Function wrapping a type into `FnWrapper` so there is better type elision or at least nicer
/// turbofish syntax
pub fn wap<Msg, F>(f: F) -> FnWrapper<F, Msg>
where
    F: WrappableFn<Msg>,
{
    f.wrap()
}

/// Default `sudo` entry point used when none is provided
pub(crate) fn default_sudo_fn<Q, C>(
    _deps: DepsMut<Q>,
    _env: Env,
    _msg: Empty,
) -> AnyResult<Response<C>>
where
    Q: CustomQuery,
{
    bail!("Sudo not implemented on the contract")
}

/// Default `reply` entry point used when none is provided
pub(crate) fn default_reply_fn<Q, C>(
    _deps: DepsMut<Q>,
    _env: Env,
    _msg: Reply,
) -> AnyResult<Response<C>>
where
    Q: CustomQuery,
{
    bail!("Reply not implemented on the contract")
}

/// Default `migrate` entry point used when none is provided
pub(crate) fn default_migrate_fn<Q, C>(
    _deps: DepsMut<Q>,
    _env: Env,
    _msg: Empty,
) -> AnyResult<Response<C>>
where
    Q: CustomQuery,
{
    bail!("Migrate not implemented on the contract")
}
