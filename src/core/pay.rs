pub const TAG: &str = "payRequest";
pub mod client;
pub mod server;

#[derive(Clone, Debug)]
pub enum Amount {
    Millisatoshis(u64),
    Currency(String, u64),
}

#[derive(Clone, Debug)]
pub struct Currency {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub multiplier: f64,
    pub convertible: Option<CurrencyConvertible>,
}

#[derive(Clone, Debug)]
pub struct CurrencyConvertible {
    pub min: u64,
    pub max: u64,
}

#[derive(Clone, Debug)]
pub struct PayerRequirements {
    pub name: Option<PayerRequirement>,
    pub pubkey: Option<PayerRequirement>,
    pub identifier: Option<PayerRequirement>,
    pub email: Option<PayerRequirement>,
    pub auth: Option<PayerRequirementAuth>,
    pub others: std::collections::HashMap<String, PayerRequirement>,
}

#[derive(Clone, Debug)]
pub struct PayerRequirement {
    pub mandatory: bool,
}

#[derive(Clone, Debug)]
pub struct PayerRequirementAuth {
    pub mandatory: bool,
    pub k1: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct PayerInformations {
    pub name: Option<String>,
    pub pubkey: Option<Vec<u8>>,
    pub identifier: Option<String>,
    pub email: Option<String>,
    pub auth: Option<PayerInformationAuth>,
}

#[derive(Clone, Debug)]
pub struct PayerInformationAuth {
    pub key: Vec<u8>,
    pub k1: [u8; 32],
    pub sig: [u8; 64],
}

mod serde {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize, Serialize)]
    pub(super) struct Currency<'a> {
        pub code: &'a str,
        pub name: &'a str,
        pub symbol: &'a str,
        pub decimals: u8,
        pub multiplier: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub convertible: Option<CurrencyConvertible>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct CurrencyConvertible {
        pub min: u64,
        pub max: u64,
    }

    #[derive(Deserialize, Serialize)]
    pub(super) struct Payer<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<PayerRequirement>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pubkey: Option<PayerRequirement>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub identifier: Option<PayerRequirement>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub email: Option<PayerRequirement>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<PayerRequirementAuth>,
        #[serde(borrow, flatten)]
        pub others: HashMap<&'a str, PayerRequirement>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct PayerRequirement {
        pub mandatory: bool,
    }

    #[derive(Deserialize, Serialize)]
    pub struct PayerRequirementAuth {
        pub mandatory: bool,
        #[serde(with = "hex::serde")]
        pub k1: [u8; 32],
    }

    #[derive(Deserialize, Serialize)]
    pub struct PayerInformations<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pubkey: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub identifier: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub email: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<PayerInformationAuth>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct PayerInformationAuth {
        #[serde(with = "hex::serde")]
        pub key: Vec<u8>,
        #[serde(with = "hex::serde")]
        pub k1: [u8; 32],
        #[serde(with = "hex::serde")]
        pub sig: [u8; 64],
    }

    pub(super) mod amount {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        pub fn serialize<S: Serializer>(
            a: &super::super::Amount,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            match a {
                crate::pay::Amount::Millisatoshis(a) => (*a).serialize(serializer),
                crate::pay::Amount::Currency(c, a) => format!("{a}.{c}").serialize(serializer),
            }
        }

        pub fn deserialize<'a, D: Deserializer<'a>>(
            deserializer: D,
        ) -> Result<super::super::Amount, D::Error> {
            let s = <&str>::deserialize(deserializer)?;

            let (amount, currency) = s.split_once('.').map_or((s, None), |(a, c)| (a, Some(c)));
            let amount = amount.parse::<u64>().map_err(serde::de::Error::custom)?;

            Ok(match currency {
                Some(currency) => super::super::Amount::Currency(String::from(currency), amount),
                None => super::super::Amount::Millisatoshis(amount),
            })
        }
    }
}
