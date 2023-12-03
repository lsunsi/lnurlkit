pub const TAG: &str = "channelRequest";

#[derive(Clone, Debug)]
pub struct ChannelRequest {
    callback: crate::serde::Url,
    pub uri: String,
    k1: String,
}

impl ChannelRequest {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback_accept(mut self, remoteid: &str, private: bool) -> url::Url {
        self.callback.0.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("private", if private { "1" } else { "0" }),
        ]);

        self.callback.0
    }

    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback_cancel(mut self, remoteid: &str) -> url::Url {
        self.callback.0.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("cancel", "1"),
        ]);

        self.callback.0
    }
}

impl std::str::FromStr for ChannelRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: serde::QueryResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(ChannelRequest {
            callback: d.callback,
            uri: d.uri,
            k1: d.k1,
        })
    }
}

mod serde {
    use crate::serde::Url;
    use miniserde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct QueryResponse {
        pub callback: Url,
        pub uri: String,
        pub k1: String,
    }
}
