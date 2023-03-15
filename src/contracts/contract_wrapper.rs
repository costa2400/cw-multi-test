use anyhow::Result as AnyResult;
use cosmwasm_std::{
    from_slice, Binary, CustomMsg, CustomQuery, Deps, DepsMut, Empty, Env, Reply, Response,
};

use crate::Contract;

use super::entry_points::{
    default_migrate_fn, default_reply_fn, default_sudo_fn, ContractFn, PermissionedFn, QueryFn,
    ReplyFn,
};

type DefPermissionedFn<C, Q> = fn(deps: DepsMut<Q>, env: Env, msg: Empty) -> AnyResult<Response<C>>;
type DefReplyFn<C, Q> = fn(deps: DepsMut<Q>, env: Env, msg: Reply) -> AnyResult<Response<C>>;

pub struct ContractWrapper<ExecuteFn, InstantaiteFn, QueryFn, SudoFn, ReplyFn, MigrateFn> {
    execute_fn: ExecuteFn,
    instantiate_fn: InstantaiteFn,
    query_fn: QueryFn,
    sudo_fn: SudoFn,
    reply_fn: ReplyFn,
    migrate_fn: MigrateFn,
}

impl<C, Q, ExecuteFn, InstantaiteFn, QueryFn>
    ContractWrapper<
        ExecuteFn,
        InstantaiteFn,
        QueryFn,
        DefPermissionedFn<C, Q>,
        DefReplyFn<C, Q>,
        DefPermissionedFn<C, Q>,
    >
where
    C: CustomMsg,
    Q: CustomQuery,
    Self: Contract<C, Q>,
{
    pub fn new(execute_fn: ExecuteFn, instantiate_fn: InstantaiteFn, query_fn: QueryFn) -> Self {
        Self {
            execute_fn,
            instantiate_fn,
            query_fn,
            sudo_fn: default_sudo_fn::<Q, C>,
            reply_fn: default_reply_fn,
            migrate_fn: default_migrate_fn::<Q, C>,
        }
    }
}

impl<C, Q, ExecuteFnT, InstantaiteFnT, QueryFnT, SudoFnT, ReplyFnT, MigrateFnT> Contract<C, Q>
    for ContractWrapper<ExecuteFnT, InstantaiteFnT, QueryFnT, SudoFnT, ReplyFnT, MigrateFnT>
where
    C: CustomMsg,
    Q: CustomQuery,
    ExecuteFnT: ContractFn<Q, C>,
    InstantaiteFnT: ContractFn<Q, C>,
    QueryFnT: QueryFn<Q>,
    SudoFnT: PermissionedFn<Q, C>,
    ReplyFnT: ReplyFn<Q, C>,
    MigrateFnT: PermissionedFn<Q, C>,
{
    fn execute(
        &self,
        deps: cosmwasm_std::DepsMut<Q>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Response<C>> {
        self.execute_fn.call(deps, env, info, from_slice(&msg)?)
    }

    fn instantiate(
        &self,
        deps: cosmwasm_std::DepsMut<Q>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Response<C>> {
        self.instantiate_fn.call(deps, env, info, from_slice(&msg)?)
    }

    fn query(
        &self,
        deps: cosmwasm_std::Deps<Q>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Binary> {
        self.query_fn.call(deps, env, from_slice(&msg)?)
    }

    fn sudo(
        &self,
        deps: cosmwasm_std::DepsMut<Q>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Response<C>> {
        self.sudo_fn.call(deps, env, from_slice(&msg)?)
    }

    fn reply(
        &self,
        deps: cosmwasm_std::DepsMut<Q>,
        env: cosmwasm_std::Env,
        msg: cosmwasm_std::Reply,
    ) -> anyhow::Result<cosmwasm_std::Response<C>> {
        self.reply_fn.call(deps, env, msg)
    }

    fn migrate(
        &self,
        deps: cosmwasm_std::DepsMut<Q>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Response<C>> {
        self.migrate_fn.call(deps, env, from_slice(&msg)?)
    }
}
