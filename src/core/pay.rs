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

mod serde {
    use serde::{Deserialize, Serialize};

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
