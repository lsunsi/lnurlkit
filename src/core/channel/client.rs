#[derive(Clone, Debug)]
pub struct Query {
    callback: url::Url,
    pub uri: String,
    k1: String,
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: de::Query = serde_json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(Query {
            callback: d.callback,
            uri: d.uri,
            k1: d.k1,
        })
    }
}

impl Query {
    #[must_use]
    pub fn callback_accept<'a>(&'a self, remoteid: &'a str, private: bool) -> CallbackRequest<'a> {
        CallbackRequest::Accept {
            url: &self.callback,
            k1: &self.k1,
            remoteid,
            private,
        }
    }

    #[must_use]
    pub fn callback_cancel<'a>(&'a self, remoteid: &'a str) -> CallbackRequest<'a> {
        CallbackRequest::Cancel {
            url: &self.callback,
            k1: &self.k1,
            remoteid,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CallbackRequest<'a> {
    Accept {
        url: &'a url::Url,
        remoteid: &'a str,
        k1: &'a str,
        private: bool,
    },
    Cancel {
        url: &'a url::Url,
        remoteid: &'a str,
        k1: &'a str,
    },
}

impl std::fmt::Display for CallbackRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallbackRequest::Cancel { url, remoteid, k1 } => {
                let mut url = (*url).clone();
                let query = [("k1", *k1), ("remoteid", remoteid), ("cancel", "1")];
                url.query_pairs_mut().extend_pairs(query);
                f.write_str(url.as_str())
            }
            CallbackRequest::Accept {
                url,
                remoteid,
                private,
                k1,
            } => {
                let query = [
                    ("k1", *k1),
                    ("remoteid", remoteid),
                    ("private", if *private { "1" } else { "0" }),
                ];

                let mut url = (*url).clone();
                url.query_pairs_mut().extend_pairs(query);
                f.write_str(url.as_str())
            }
        }
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
    pub(super) struct Query {
        pub callback: Url,
        pub uri: String,
        pub k1: String,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn query_parse() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed = input.parse::<super::Query>().expect("parse");

        assert_eq!(parsed.callback.as_str(), "https://yuri/?o=callback");
        assert_eq!(parsed.uri, "noh@ipe:porta");
        assert_eq!(parsed.k1, "caum");
    }

    #[test]
    fn callback_request_accept_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed = input.parse::<super::Query>().expect("parse");
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
    fn callback_request_cancel_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed = input.parse::<super::Query>().expect("parse");
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
