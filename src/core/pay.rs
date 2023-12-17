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
    pub convertible: bool,
}

#[derive(Clone, Debug)]
pub struct Payer {
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
        #[serde(default)]
        pub convertible: bool,
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
