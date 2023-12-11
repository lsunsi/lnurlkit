pub const TAG: &str = "withdrawRequest";
pub mod client;
pub mod server;

mod serde {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub(super) struct CallbackQuery<'a> {
        pub k1: &'a str,
        pub pr: &'a str,
    }
}
