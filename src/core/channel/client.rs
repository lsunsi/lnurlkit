#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub callback: url::Url,
    pub uri: String,
    pub k1: String,
}

impl TryFrom<&[u8]> for Entrypoint {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let d: de::Entrypoint = serde_json::from_slice(s).map_err(|_| "deserialize failed")?;

        Ok(Entrypoint {
            callback: d.callback,
            uri: d.uri,
            k1: d.k1,
        })
    }
}

impl Entrypoint {
    #[must_use]
    pub fn accept<'a>(&'a self, remoteid: &'a str, private: bool) -> Callback<'a> {
        Callback::Accept {
            url: &self.callback,
            k1: &self.k1,
            remoteid,
            private,
        }
    }

    #[must_use]
    pub fn cancel<'a>(&'a self, remoteid: &'a str) -> Callback<'a> {
        Callback::Cancel {
            url: &self.callback,
            k1: &self.k1,
            remoteid,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Callback<'a> {
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

impl std::fmt::Display for Callback<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (url, query) = match self {
            Callback::Accept {
                url,
                k1,
                remoteid,
                private,
            } => (
                url,
                super::serde::Callback::Accept {
                    k1,
                    remoteid,
                    private: if *private {
                        super::serde::ZeroOrOne::One
                    } else {
                        super::serde::ZeroOrOne::Zero
                    },
                },
            ),
            Callback::Cancel { url, k1, remoteid } => (
                url,
                super::serde::Callback::Cancel {
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

mod de {
    use serde::Deserialize;
    use url::Url;

    #[derive(Deserialize)]
    pub(super) struct Entrypoint {
        pub callback: Url,
        pub uri: String,
        pub k1: String,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entrypoint_parse() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(parsed.callback.as_str(), "https://yuri/?o=callback");
        assert_eq!(parsed.uri, "noh@ipe:porta");
        assert_eq!(parsed.k1, "caum");
    }

    #[test]
    fn callback_accept_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        let url = parsed.accept("idremoto", true);

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&private=1"
        );

        let url = parsed.accept("idremoto", false);

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&private=0"
        );
    }

    #[test]
    fn callback_cancel_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "uri": "noh@ipe:porta",
            "k1": "caum"
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        let url = parsed.cancel("idremoto");

        assert_eq!(
            url.to_string(),
            "https://yuri/?o=callback&k1=caum&remoteid=idremoto&cancel=1"
        );
    }
}
