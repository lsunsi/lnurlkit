#[derive(Clone, Debug)]
pub struct Query {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&miniserde::json::to_string(&ser::Query {
            tag: super::TAG,
            callback: crate::serde::Url(std::borrow::Cow::Borrowed(&self.callback)),
            uri: &self.uri,
            k1: &self.k1,
        }))
    }
}

#[derive(Clone, Debug)]
pub enum CallbackRequest {
    Accept {
        remoteid: Box<str>,
        k1: Box<str>,
        private: bool,
    },
    Cancel {
        remoteid: Box<str>,
        k1: Box<str>,
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
                remoteid: (*remoteid).into(),
                k1: (*k1).into(),
            })
        } else {
            match qs.get("private").copied() {
                Some("0") => Some(CallbackRequest::Accept {
                    remoteid: (*remoteid).into(),
                    k1: (*k1).into(),
                    private: false,
                }),
                Some("1") => Some(CallbackRequest::Accept {
                    remoteid: (*remoteid).into(),
                    k1: (*k1).into(),
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

        assert_eq!(&remoteid as &str, "idremoto");
        assert_eq!(&k1 as &str, "caum");
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

        assert_eq!(&remoteid as &str, "idremoto");
        assert_eq!(&k1 as &str, "caum");
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

        assert_eq!(&remoteid as &str, "idremoto");
        assert_eq!(&k1 as &str, "caum");

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
