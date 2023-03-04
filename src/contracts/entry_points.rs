//! Traits implemented for anything which can be an entry point, so it is aligned to inner
//! multi-test structure. They are defined for any function-like object with matching signatures,
//! being extension traits for `Fn`

use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{Binary, CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response};

/// `execute` or `instantiate` entry point
///
/// * `Msg` - a message the entry point is handling
/// * `Q` - a blockchain-specific query-type
/// * `C` - a blockchain-specific custom-type
pub trait ContractFn<Msg, Q, C>
where
    Q: CustomQuery,
{
    fn call(
        &self,
        deps: DepsMut<Q>,
        env: Env,
        info: MessageInfo,
        msg: Msg,
    ) -> AnyResult<Response<C>>;
}

impl<F, T, Q, C, E> ContractFn<T, Q, C> for F
where
    F: Fn(DepsMut<Q>, Env, MessageInfo, T) -> Result<Response<C>, E>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
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
/// * `Msg` - a message the entry point is handling
/// * `Q` - a blockchain-specific query-type
/// * `C` - a blockchain-specific custom-type
pub trait PermissionedFn<Msg, Q, C>
where
    Q: CustomQuery,
{
    fn call(&self, deps: DepsMut<Q>, env: Env, msg: Msg) -> AnyResult<Response<C>>;
}

impl<F, T, Q, C, E> PermissionedFn<T, Q, C> for F
where
    F: Fn(DepsMut<Q>, Env, T) -> Result<Response<C>, E>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    fn call(&self, deps: DepsMut<Q>, env: Env, msg: T) -> AnyResult<Response<C>> {
        self(deps, env, msg).map_err(Into::into)
    }
}

/// `reply` entry point
///
/// * `Msg` - a message the entry point is handling
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

/// `query` entry point
///
/// * `Msg` - a message the entry point is handling
/// * `Q` - a blockchain-specific query-type
/// * `C` - a blockchain-specific custom-type
pub trait QueryFn<Msg, Q>
where
    Q: CustomQuery,
{
    fn call(&self, deps: Deps<Q>, env: Env, msg: Msg) -> AnyResult<Binary>;
}

impl<F, T, Q, E> QueryFn<T, Q> for F
where
    F: Fn(Deps<Q>, Env, T) -> Result<Binary, E>,
    E: Into<anyhow::Error>,
    Q: CustomQuery,
{
    fn call(&self, deps: Deps<Q>, env: Env, msg: T) -> AnyResult<Binary> {
        self(deps, env, msg).map_err(Into::into)
    }
}

/// Default `sudo` entry point used when none is provided
fn default_sudo_fn<Q, C>(
    _deps: DepsMut<Q>,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> AnyResult<Response<C>>
where
    Q: CustomQuery,
{
    bail!("Sudo not implemented on the contract")
}

/// Default `reply` entry point used when none is provided
fn default_reply_fn<Q, C>(_deps: DepsMut<Q>, _env: Env, _msg: Reply) -> AnyResult<Response<C>>
where
    Q: CustomQuery,
{
    bail!("Reply not implemented on the contract")
}

/// Default `migrate` entry point used when none is provided
fn default_migrate_fn<Q, C>(
    _deps: DepsMut<Q>,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> AnyResult<Response<C>>
where
    Q: CustomQuery,
{
    bail!("Migrate not implemented on the contract")
}
