pub const TAG: &str = "channelRequest";
pub mod client;
pub mod server;

mod serde {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    #[serde(untagged)]
    pub(super) enum Callback<'a> {
        Accept {
            k1: &'a str,
            remoteid: &'a str,
            private: ZeroOrOne,
        },
        Cancel {
            k1: &'a str,
            remoteid: &'a str,
            cancel: OneOnly,
        },
    }

    #[derive(Deserialize, Serialize)]
    pub(super) enum OneOnly {
        #[serde(rename = "1")]
        One,
    }

    #[derive(Deserialize, Serialize)]
    pub(super) enum ZeroOrOne {
        #[serde(rename = "0")]
        Zero,
        #[serde(rename = "1")]
        One,
    }
}
