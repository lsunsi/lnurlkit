pub const TAG: &str = "payRequest";
pub mod client;
pub mod server;

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

    #[derive(Deserialize, Serialize)]
    pub(super) struct Callback<'a> {
        pub comment: Option<&'a str>,
        pub amount: u64,
    }
}
