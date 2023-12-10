#[derive(Clone, Debug)]
pub struct Query {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = serde_json::to_string(&ser::Query {
            tag: super::TAG,
            callback: &self.callback,
            uri: &self.uri,
            k1: &self.k1,
        });

        f.write_str(&ser.map_err(|_| std::fmt::Error)?)
    }
}

#[derive(Clone, Debug)]
pub enum CallbackRequest {
    Accept {
        remoteid: String,
        k1: String,
        private: bool,
    },
    Cancel {
        remoteid: String,
        k1: String,
    },
}

impl std::str::FromStr for CallbackRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let qs = s
            .split('&')
            .filter_map(|s| s.split_once('='))
            .collect::<std::collections::HashMap<_, _>>();

        let k1 = qs.get("k1").ok_or("missing k1")?;
        let remoteid = qs.get("remoteid").ok_or("missing remoteid")?;

        if qs.get("cancel").copied() == Some("1") {
            Some(CallbackRequest::Cancel {
                remoteid: String::from(*remoteid),
                k1: String::from(*k1),
            })
        } else {
            match qs.get("private").copied() {
                Some("0") => Some(CallbackRequest::Accept {
                    remoteid: String::from(*remoteid),
                    k1: String::from(*k1),
                    private: false,
                }),
                Some("1") => Some(CallbackRequest::Accept {
                    remoteid: String::from(*remoteid),
                    k1: String::from(*k1),
                    private: true,
                }),
                _ => None,
            }
        }
        .ok_or("missing cancel/private")
    }
}

#[derive(Clone, Debug)]
pub enum CallbackResponse {
    Error { reason: String },
    Ok,
}

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = std::collections::BTreeMap::new();

        match self {
            CallbackResponse::Error { reason } => {
                map.insert("status", "ERROR");
                map.insert("reason", reason);
            }
            CallbackResponse::Ok => {
                map.insert("status", "OK");
            }
        }

        f.write_str(&serde_json::to_string(&map).map_err(|_| std::fmt::Error)?)
    }
}

mod ser {
    use serde::Serialize;
    use url::Url;

    #[derive(Serialize)]
    pub(super) struct Query<'a> {
        pub tag: &'static str,
        pub callback: &'a Url,
        pub uri: &'a str,
        pub k1: &'a str,
    }
}

#[cfg(test)]
mod tests {
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
        let input = "remoteid=idremoto&k1=caum&private=1";
        let parsed = input.parse::<super::CallbackRequest>().expect("parse");

        let super::CallbackRequest::Accept {
            remoteid,
            private,
            k1,
        } = parsed
        else {
            panic!("wrong parsed");
        };

        assert_eq!(remoteid, "idremoto");
        assert_eq!(k1, "caum");
        assert!(private);

        let input = "remoteid=idremoto&k1=caum&private=0";
        let parsed = input.parse::<super::CallbackRequest>().expect("parse");

        let super::CallbackRequest::Accept {
            remoteid,
            private,
            k1,
        } = parsed
        else {
            panic!("wrong parsed");
        };

        assert_eq!(remoteid, "idremoto");
        assert_eq!(k1, "caum");
        assert!(!private);

        let input = "remoteid=idremoto&k1=caum&private=2";
        let parsed = input.parse::<super::CallbackRequest>();
        assert!(parsed.is_err());
    }

    #[test]
    fn callback_request_cancel_parse() {
        let input = "remoteid=idremoto&k1=caum&cancel=1";
        let parsed = input.parse::<super::CallbackRequest>().expect("parse");

        let super::CallbackRequest::Cancel { remoteid, k1 } = parsed else {
            panic!("wrong parsed");
        };

        assert_eq!(remoteid, "idremoto");
        assert_eq!(k1, "caum");

        let input = "remoteid=idremoto&k1=caum&cancel=0";
        let parsed = input.parse::<super::CallbackRequest>();
        assert!(parsed.is_err());
    }

    #[test]
    fn callback_response_render() {
        assert_eq!(
            super::CallbackResponse::Ok.to_string(),
            r#"{"status":"OK"}"#
        );

        assert_eq!(
            super::CallbackResponse::Error {
                reason: String::from("razao")
            }
            .to_string(),
            r#"{"reason":"razao","status":"ERROR"}"#
        );
    }
}
