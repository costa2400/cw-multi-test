//! Extension traits for "contextual" or "common" cosmwams-std types allowing for proper conversion
//! of their counterparts on the other chain.

use cosmwasm_std::{
    CosmosMsg, CustomQuery, Deps, DepsMut, Empty, QuerierWrapper, Response, SubMsg,
};

/// Trait converting `DepsMut` to one operating on another `Query` type. By default only
/// conversions from any `DepsMut<Q>` to `DepsMut<Empty>` are possible, and in general - only
/// converting to `DepsMut` over simpler query (being a subset of the original one) should be
/// allowed.
pub trait CustomizeDepsMut<'deps, Q>
where
    Q: CustomQuery,
{
    fn customize(self) -> DepsMut<'deps, Q>;
}

/// Any `DepsMut<Q>` can be made into `DepsMut<Empty>`
///
/// It would be better to define it on owned `DepsMut`, but the `QuerierWrapper::querier` is not
/// accessible - some destructuring function for it would be helpfull here
impl<'deps, Q> CustomizeDepsMut<'deps, Empty> for &'deps mut DepsMut<'deps, Q>
where
    Q: CustomQuery,
{
    fn customize(self) -> DepsMut<'deps, Empty> {
        DepsMut {
            storage: self.storage,
            api: self.api,
            querier: QuerierWrapper::new(&*self.querier),
        }
    }
}

/// Trait converting `Deps` to one operating on another `Query` type. By default only conversions
/// from any `Deps<Q>` to `Deps<Empty>` are possible, and in general - only converting to `Deps`
/// over simpler query (being a subset of the original one) should be allowed.
pub trait CustomizeDeps<'deps, Q>
where
    Q: CustomQuery,
{
    fn customize(self) -> Deps<'deps, Q>;
}

/// Any `Deps<Q>` can be made into `Deps<Empty>`
///
/// It would be better to define it on owned `Deps`, but the `QuerierWrapper::querier` is not
/// accessible - some destructuring function for it would be helpfull here
impl<'deps, Q> CustomizeDeps<'deps, Empty> for &'deps Deps<'deps, Q>
where
    Q: CustomQuery,
{
    fn customize(self) -> Deps<'deps, Empty> {
        Deps {
            storage: self.storage,
            api: self.api,
            querier: QuerierWrapper::new(&*self.querier),
        }
    }
}

/// Trait converting `SubMsg` to one carrying another chain-custom message
pub trait CustomizeMsg<C> {
    fn customize(self) -> SubMsg<C>;
}

/// `SubMsg<Empty>` can be made into any `SubMsg<Q>`
impl<C> CustomizeMsg<C> for SubMsg<Empty> {
    fn customize(self) -> SubMsg<C> {
        SubMsg {
            msg: match self.msg {
                CosmosMsg::Wasm(wasm) => CosmosMsg::Wasm(wasm),
                CosmosMsg::Bank(bank) => CosmosMsg::Bank(bank),
                CosmosMsg::Staking(staking) => CosmosMsg::Staking(staking),
                CosmosMsg::Distribution(distribution) => CosmosMsg::Distribution(distribution),
                CosmosMsg::Custom(_) => unreachable!(),
                #[cfg(feature = "stargate")]
                CosmosMsg::Ibc(ibc) => CosmosMsg::Ibc(ibc),
                #[cfg(feature = "stargate")]
                CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
                _ => panic!("unknown message variant {:?}", self),
            },
            id: self.id,
            gas_limit: self.gas_limit,
            reply_on: self.reply_on,
        }
    }
}

/// Trait converting `Response` to one carrying another chain-custom messages
pub trait CustomizeResponse<C> {
    fn customize(self) -> Response<C>;
}

/// `Response<Empty>` can be made into any `Response<Q>`
impl<C> CustomizeResponse<C> for Response<Empty> {
    fn customize(self) -> Response<C> {
        let mut resp = Response::new()
            .add_submessages(self.messages.into_iter().map(CustomizeMsg::customize))
            .add_events(self.events)
            .add_attributes(self.attributes);
        resp.data = self.data;

        resp
    }
}
