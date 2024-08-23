use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::{IpNetwork, IpNetworkError, Ipv4Network, Ipv6Network};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::unexpected_nonnull_type;
use crate::bindings::durable::core::sql;
use crate::driver::{Durable, TypeInfo, Value};

impl sqlx::Encode<'_, Durable> for IpNetwork {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value(sql::Value::inet((*self).into())?));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Ipv4Network {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <IpNetwork as sqlx::Encode<Durable>>::encode(IpNetwork::V4(*self), buf)
    }
}

impl sqlx::Encode<'_, Durable> for Ipv6Network {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <IpNetwork as sqlx::Encode<Durable>>::encode(IpNetwork::V6(*self), buf)
    }
}

impl sqlx::Decode<'_, Durable> for IpNetwork {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(inet) = value.0.as_inet() {
            return Ok(inet.try_into()?);
        }

        Err(unexpected_nonnull_type("inet", value))
    }
}

impl sqlx::Type<Durable> for IpNetwork {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::inet()
    }
}

impl sqlx::Type<Durable> for Ipv4Network {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::inet()
    }
}

impl sqlx::Type<Durable> for Ipv6Network {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::inet()
    }
}

impl sqlx::Encode<'_, Durable> for IpAddr {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let inet = match *self {
            Self::V4(v4) => IpNetwork::V4(Ipv4Network::new(v4, 32)?),
            Self::V6(v6) => IpNetwork::V6(Ipv6Network::new(v6, 128)?),
        };

        <IpNetwork as sqlx::Encode<Durable>>::encode(inet, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Ipv4Addr {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let inet = IpNetwork::V4(Ipv4Network::new(*self, 32)?);

        <IpNetwork as sqlx::Encode<Durable>>::encode(inet, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Ipv6Addr {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let inet = IpNetwork::V6(Ipv6Network::new(*self, 128)?);

        <IpNetwork as sqlx::Encode<Durable>>::encode(inet, buf)
    }
}

impl sqlx::Decode<'_, Durable> for IpAddr {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        let inet = <IpNetwork as sqlx::Decode<Durable>>::decode(value)?;
        let addr = inet.ip();

        let has_maximal_prefix = match inet {
            IpNetwork::V4(_) => inet.prefix() == 32,
            IpNetwork::V6(_) => inet.prefix() == 128,
        };

        if !has_maximal_prefix {
            return Err(
                "attempted to decode an IpAddr from a ip network containing multiple addresses"
                    .into(),
            );
        }

        Ok(addr)
    }
}

impl sqlx::Type<Durable> for IpAddr {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::inet()
    }
}

impl From<IpNetwork> for sql::IpNetwork {
    fn from(value: IpNetwork) -> Self {
        match value {
            IpNetwork::V4(v4) => sql::IpNetwork::V4(sql::Ipv4Network {
                addr: v4.ip().to_bits(),
                prefix: v4.prefix(),
            }),
            IpNetwork::V6(v6) => {
                let bits = v6.ip().to_bits();
                let lo = bits as u64;
                let hi = (bits >> 64) as u64;

                sql::IpNetwork::V6(sql::Ipv6Network {
                    addr: (lo, hi),
                    prefix: v6.prefix(),
                })
            }
        }
    }
}

impl TryFrom<sql::IpNetwork> for IpNetwork {
    type Error = IpNetworkError;

    fn try_from(value: sql::IpNetwork) -> Result<Self, Self::Error> {
        Ok(match value {
            sql::IpNetwork::V4(v4) => {
                Self::V4(Ipv4Network::new(Ipv4Addr::from_bits(v4.addr), v4.prefix)?)
            }
            sql::IpNetwork::V6(v6) => {
                let addr = (v6.addr.0 as u128) | (v6.addr.1 as u128) << 64;

                Self::V6(Ipv6Network::new(Ipv6Addr::from_bits(addr), v6.prefix)?)
            }
        })
    }
}
