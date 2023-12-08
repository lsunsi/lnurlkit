pub const TAG: &str = "channelRequest";

#[derive(Clone, Debug)]
pub struct Query {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: de::Query = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(Query {
            callback: d.callback.0.into_owned(),
            uri: d.uri,
            k1: d.k1,
        })
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&miniserde::json::to_string(&ser::Query {
            tag: TAG,
            callback: crate::serde::Url(std::borrow::Cow::Borrowed(&self.callback)),
            uri: &self.uri,
            k1: &self.k1,
        }))
    }
}

impl Query {
    #[must_use]
    pub fn callback_accept(self, remoteid: String, private: bool) -> CallbackRequest {
        CallbackRequest::Accept {
            url: self.callback,
            k1: self.k1,
            remoteid,
            private,
        }
    }

    #[must_use]
    pub fn callback_cancel(self, remoteid: String) -> CallbackRequest {
        CallbackRequest::Cancel {
            url: self.callback,
            k1: self.k1,
            remoteid,
        }
    }
}

pub enum CallbackRequest {
    Cancel {
        url: url::Url,
        remoteid: String,
        k1: String,
    },
    Accept {
        url: url::Url,
        remoteid: String,
        private: bool,
        k1: String,
    },
}

impl CallbackRequest {
    #[must_use]
    pub fn url(self) -> url::Url {
        match self {
            CallbackRequest::Cancel {
                mut url,
                remoteid,
                k1,
            } => {
                let query = [
                    ("k1", k1),
                    ("remoteid", remoteid),
                    ("cancel", String::from("1")),
                ];

                url.query_pairs_mut().extend_pairs(query);
                url
            }
            CallbackRequest::Accept {
                mut url,
                remoteid,
                private,
                k1,
            } => {
                let query = [
                    ("k1", k1),
                    ("remoteid", remoteid),
                    ("private", String::from(if private { "1" } else { "0" })),
                ];

                url.query_pairs_mut().extend_pairs(query);
                url
            }
        }
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
    pub(super) struct Query<'a> {
        pub tag: &'static str,
        pub callback: Url<'a>,
        pub uri: &'a str,
        pub k1: &'a str,
    }
}

mod de {
    use crate::serde::Url;
    use miniserde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct Query {
        pub callback: Url<'static>,
        pub uri: String,
        pub k1: String,
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    #[test]
    fn query_parse() {
        let input = r#"
            {
                "uri": "noh@ipe:porta",
                "callback": "https://yuri?o=callback",
                "k1": "caum"
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.uri, "noh@ipe:porta");
        assert_eq!(parsed.k1, "caum");
    }

    #[test]
    fn query_render() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri/?o=callback").expect("url"),
            uri: String::from("noh@ipe:porta"),
            k1: String::from("caum"),
        };

        let json = r#"{"tag":"channelRequest","callback":"https://yuri/?o=callback","uri":"noh@ipe:porta","k1":"caum"}"#;
        assert_eq!(query.to_string(), json);
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

        let parsed = input.parse::<super::Query>().expect("parse");
        let url = parsed
            .clone()
            .callback_accept(String::from("idremoto"), true)
            .url();

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&private=1"
        );

        let url = parsed
            .callback_accept(String::from("idremoto"), false)
            .url();

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

        let parsed = input.parse::<super::Query>().expect("parse");
        let url = parsed.callback_cancel(String::from("idremoto")).url();

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&cancel=1"
        );
    }

    #[test]
    fn callback_response_parse() {
        assert!(matches!(
            r#"{ "status": "OK"}"#.parse().unwrap(),
            super::CallbackResponse::Ok
        ));

        assert!(matches!(
            r#"{ "status": "ERROR", "reason": "razao" }"#.parse().unwrap(),
            super::CallbackResponse::Error(r) if r == "razao"
        ));
    }

    #[test]
    fn callback_response_render() {
        assert_eq!(
            super::CallbackResponse::Ok.to_string(),
            r#"{"status":"OK"}"#
        );
        assert_eq!(
            super::CallbackResponse::Error(String::from("razao")).to_string(),
            r#"{"reason":"razao","status":"ERROR"}"#
        );
    }
}
