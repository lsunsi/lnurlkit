pub const TAG: &str = "channelRequest";

#[derive(Clone, Debug)]
pub struct ChannelRequest {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl std::str::FromStr for ChannelRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: de::QueryResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(ChannelRequest {
            callback: d.callback.0,
            uri: d.uri,
            k1: d.k1,
        })
    }
}

impl std::fmt::Display for ChannelRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&miniserde::json::to_string(&ser::QueryResponse {
            tag: TAG,
            callback: crate::serde::Url(self.callback.clone()),
            uri: &self.uri,
            k1: &self.k1,
        }))
    }
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

#[derive(Debug)]
pub enum CallbackAction {
    Accept { private: bool },
    Cancel,
}

#[derive(Debug)]
pub enum CallbackResponse {
    Error(String),
    Ok,
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = miniserde::json::from_str::<std::collections::BTreeMap<String, String>>(s)
            .map_err(|_| "bad json")?;

        match map.get("status").map(|s| s as &str) {
            Some("OK") => Ok(CallbackResponse::Ok),
            Some("ERROR") => Ok(CallbackResponse::Error(
                map.get("reason").cloned().unwrap_or_default(),
            )),
            _ => Err("bad status field"),
        }
    }
}

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = std::collections::BTreeMap::new();

        match self {
            CallbackResponse::Error(reason) => {
                map.insert("status", "ERROR");
                map.insert("reason", reason);
            }
            CallbackResponse::Ok => {
                map.insert("status", "OK");
            }
        }

        f.write_str(&miniserde::json::to_string(&map))
    }
}

mod ser {
    use crate::serde::Url;
    use miniserde::Serialize;

    #[derive(Serialize)]
    pub(super) struct QueryResponse<'a> {
        pub tag: &'static str,
        pub callback: Url,
        pub uri: &'a str,
        pub k1: &'a str,
    }
}

mod de {
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
