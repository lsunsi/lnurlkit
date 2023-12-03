pub const TAG: &str = "channelRequest";

#[derive(Clone, Debug)]
pub struct ChannelRequest {
    callback: url::Url,
    pub uri: String,
    k1: String,
}

impl ChannelRequest {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback_accept(mut self, remoteid: &str, private: bool) -> url::Url {
        self.callback.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("private", if private { "1" } else { "0" }),
        ]);

        self.callback
    }

    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback_cancel(mut self, remoteid: &str) -> url::Url {
        self.callback.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("cancel", "1"),
        ]);

        self.callback
    }
}

impl std::str::FromStr for ChannelRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: serde::QueryResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(ChannelRequest {
            callback: d.callback.0,
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

#[cfg(test)]
mod tests {
    #[test]
    fn parse() {
        let input = r#"
            {
                "uri": "noh@ipe:porta",
                "callback": "https://yuri?o=callback",
                "k1": "caum"
            }
        "#;

        let parsed = input.parse::<super::ChannelRequest>().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.uri, "noh@ipe:porta");
        assert_eq!(parsed.k1, "caum");
    }

    #[test]
    fn callback_accept() {
        let input = r#"
            {
                "uri": "noh@ipe:porta",
                "callback": "https://yuri?o=callback",
                "k1": "caum"
            }
        "#;

        let parsed = input.parse::<super::ChannelRequest>().expect("parse");
        let url = parsed.clone().callback_accept("idremoto", true);

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&private=1"
        );

        let url = parsed.callback_accept("idremoto", false);

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&private=0"
        );
    }

    #[test]
    fn callback_cancel() {
        let input = r#"
            {
                "uri": "noh@ipe:porta",
                "callback": "https://yuri?o=callback",
                "k1": "caum"
            }
        "#;

        let parsed = input.parse::<super::ChannelRequest>().expect("parse");
        let url = parsed.callback_cancel("idremoto");

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&cancel=1"
        );
    }
}
