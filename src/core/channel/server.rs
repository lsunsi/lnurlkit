#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl TryFrom<Entrypoint> for Vec<u8> {
    type Error = &'static str;

    fn try_from(r: Entrypoint) -> Result<Self, Self::Error> {
        serde_json::to_vec(&ser::Entrypoint {
            tag: super::TAG,
            callback: &r.callback,
            uri: &r.uri,
            k1: &r.k1,
        })
        .map_err(|_| "serialize failed")
    }
}

#[derive(Clone, Debug)]
pub enum Callback {
    Accept {
        k1: String,
        remoteid: String,
        private: bool,
    },
    Cancel {
        k1: String,
        remoteid: String,
    },
}

impl<'a> TryFrom<&'a str> for Callback {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        serde_urlencoded::from_str::<super::serde::Callback>(s)
            .map_err(|_| "deserialize failed")
            .map(|query| match query {
                super::serde::Callback::Accept {
                    k1,
                    remoteid,
                    private,
                } => Callback::Accept {
                    k1: String::from(k1),
                    remoteid: String::from(remoteid),
                    private: match private {
                        super::serde::ZeroOrOne::Zero => false,
                        super::serde::ZeroOrOne::One => true,
                    },
                },
                super::serde::Callback::Cancel {
                    k1,
                    remoteid,
                    cancel: _,
                } => Callback::Cancel {
                    k1: String::from(k1),
                    remoteid: String::from(remoteid),
                },
            })
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
    pub(super) struct Entrypoint<'a> {
        pub tag: &'static str,
        pub callback: &'a Url,
        pub uri: &'a str,
        pub k1: &'a str,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entrypoint_render() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri/?o=callback").expect("url"),
            uri: String::from("noh@ipe:porta"),
            k1: String::from("caum"),
        };

        let json = br#"{"tag":"channelRequest","callback":"https://yuri/?o=callback","uri":"noh@ipe:porta","k1":"caum"}"#;
        assert_eq!(Vec::<u8>::try_from(query).unwrap(), json);
    }

    #[test]
    fn callback_accept_parse() {
        let input = "remoteid=idremoto&k1=caum&private=1";
        let parsed: super::Callback = input.try_into().expect("parse");

        let super::Callback::Accept {
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
        let parsed: super::Callback = input.try_into().expect("parse");

        let super::Callback::Accept {
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
        let parsed: Result<super::Callback, _> = input.try_into();
        assert!(parsed.is_err());
    }

    #[test]
    fn callback_cancel_parse() {
        let input = "remoteid=idremoto&k1=caum&cancel=1";
        let parsed: super::Callback = input.try_into().expect("parse");

        let super::Callback::Cancel { remoteid, k1 } = parsed else {
            panic!("wrong parsed");
        };

        assert_eq!(remoteid, "idremoto");
        assert_eq!(k1, "caum");

        let input = "remoteid=idremoto&k1=caum&cancel=0";
        let parsed: Result<super::Callback, _> = input.try_into();
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
