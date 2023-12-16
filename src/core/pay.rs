pub const TAG: &str = "payRequest";
pub mod client;
pub mod server;

mod serde {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub(super) struct Callback<'a> {
        pub comment: Option<&'a str>,
        pub amount: u64,
    }
}
