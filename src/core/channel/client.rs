#[derive(Clone, Debug)]
pub struct Response {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl TryFrom<&[u8]> for Response {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let d: de::Response = serde_json::from_slice(s).map_err(|_| "deserialize failed")?;

        Ok(Response {
            callback: d.callback,
            uri: d.uri,
            k1: d.k1,
        })
    }
}

impl Response {
    #[must_use]
    pub fn callback_accept<'a>(&'a self, remoteid: &'a str, private: bool) -> CallbackQuery<'a> {
        CallbackQuery::Accept {
            url: &self.callback,
            k1: &self.k1,
            remoteid,
            private,
        }
    }

    #[must_use]
    pub fn callback_cancel<'a>(&'a self, remoteid: &'a str) -> CallbackQuery<'a> {
        CallbackQuery::Cancel {
            url: &self.callback,
            k1: &self.k1,
            remoteid,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CallbackQuery<'a> {
    Accept {
        url: &'a url::Url,
        k1: &'a str,
        remoteid: &'a str,
        private: bool,
    },
    Cancel {
        url: &'a url::Url,
        k1: &'a str,
        remoteid: &'a str,
    },
}

impl std::fmt::Display for CallbackQuery<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (url, query) = match self {
            CallbackQuery::Accept {
                url,
                k1,
                remoteid,
                private,
            } => (
                url,
                super::serde::CallbackQuery::Accept {
                    k1,
                    remoteid,
                    private: if *private {
                        super::serde::ZeroOrOne::One
                    } else {
                        super::serde::ZeroOrOne::Zero
                    },
                },
            ),
            CallbackQuery::Cancel { url, k1, remoteid } => (
                url,
                super::serde::CallbackQuery::Cancel {
                    k1,
                    remoteid,
                    cancel: super::serde::OneOnly::One,
                },
            ),
        };

        let querystr = serde_urlencoded::to_string(query).map_err(|_| std::fmt::Error)?;
        let sep = if url.query().is_some() { '&' } else { '?' };
        write!(f, "{url}{sep}{querystr}")
    }
}

#[derive(Clone, Debug)]
pub enum CallbackResponse {
    Error { reason: String },
    Ok,
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = serde_json::from_str::<std::collections::BTreeMap<String, String>>(s)
            .map_err(|_| "bad json")?;

        match map.get("status").map(|s| s as &str) {
            Some("OK") => Ok(CallbackResponse::Ok),
            Some("ERROR") => {
                let reason = String::from(map.get("reason").ok_or("error without reason")?);
                Ok(CallbackResponse::Error { reason })
            }
            _ => Err("bad status field"),
        }
    }
}

mod de {
    use serde::Deserialize;
    use url::Url;

    #[derive(Deserialize)]
    pub(super) struct Response {
        pub callback: Url,
        pub uri: String,
        pub k1: String,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn response_parse() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed: super::Response = input.as_bytes().try_into().expect("parse");

        assert_eq!(parsed.callback.as_str(), "https://yuri/?o=callback");
        assert_eq!(parsed.uri, "noh@ipe:porta");
        assert_eq!(parsed.k1, "caum");
    }

    #[test]
    fn callback_query_accept_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed: super::Response = input.as_bytes().try_into().expect("parse");
        let url = parsed.callback_accept("idremoto", true);

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
    fn callback_query_cancel_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed: super::Response = input.as_bytes().try_into().expect("parse");
        let url = parsed.callback_cancel("idremoto");

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&cancel=1"
        );
    }

    #[test]
    fn callback_response_parse() {
        assert!(matches!(
            r#"{ "status": "OK" }"#.parse().unwrap(),
            super::CallbackResponse::Ok
        ));

        assert!(matches!(
            r#"{ "status": "ERROR", "reason": "razao" }"#.parse().unwrap(),
            super::CallbackResponse::Error { reason } if reason == "razao"
        ));
    }
}
