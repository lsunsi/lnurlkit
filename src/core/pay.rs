pub const TAG: &str = "payRequest";

#[derive(Clone, Debug)]
pub struct Query {
    pub callback: url::Url,
    pub short_description: String,
    pub long_description: Option<String>,
    pub identifier: Option<String>,
    pub email: Option<String>,
    pub jpeg: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
    pub comment_size: u64,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use base64::{prelude::BASE64_STANDARD, Engine};
        use miniserde::json::Value;

        let p: de::Query = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        let comment_size = p.comment_allowed.unwrap_or(0);

        let metadata = miniserde::json::from_str::<Vec<(String, Value)>>(&p.metadata)
            .map_err(|_| "deserialize metadata failed")?;

        let short_description = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/plain").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            })
            .ok_or("short description failed")?;

        let long_description = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/long-desc").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            });

        let jpeg = metadata
            .iter()
            .find_map(|(k, v)| (k == "image/jpeg;base64").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => BASE64_STANDARD.decode(s).ok(),
                _ => None,
            });

        let png = metadata
            .iter()
            .find_map(|(k, v)| (k == "image/png;base64").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => BASE64_STANDARD.decode(s).ok(),
                _ => None,
            });

        let identifier = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/identifier").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            });

        let email = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/email").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            });

        Ok(Query {
            callback: p.callback.0.into_owned(),
            min: p.min_sendable,
            max: p.max_sendable,
            short_description,
            long_description,
            comment_size,
            identifier,
            email,
            jpeg,
            png,
        })
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use base64::{prelude::BASE64_STANDARD, Engine};

        let metadata = miniserde::json::to_string(
            &[
                Some(("text/plain", self.short_description.clone())),
                self.long_description
                    .as_ref()
                    .map(|s| ("text/long-desc", s.clone())),
                self.jpeg
                    .as_ref()
                    .map(|s| ("image/jpeg;base64", BASE64_STANDARD.encode(s))),
                self.png
                    .as_ref()
                    .map(|s| ("image/png;base64", BASE64_STANDARD.encode(s))),
                self.identifier
                    .as_ref()
                    .map(|s| ("text/identifier", s.clone())),
                self.email.as_ref().map(|s| ("text/email", s.clone())),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        );

        f.write_str(&miniserde::json::to_string(&ser::Query {
            tag: TAG,
            metadata,
            callback: crate::serde::Url(std::borrow::Cow::Borrowed(&self.callback)),
            min_sendable: self.min,
            max_sendable: self.max,
            comment_allowed: self.comment_size,
        }))
    }
}

impl Query {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback(mut self, comment: &str, millisatoshis: u64) -> url::Url {
        self.callback.query_pairs_mut().extend_pairs(
            [
                (!comment.is_empty()).then_some(("comment", comment)),
                Some(("amount", &millisatoshis.to_string())),
            ]
            .into_iter()
            .flatten(),
        );

        self.callback
    }
}

#[derive(Debug)]
pub struct CallbackResponse {
    pub pr: String,
    pub disposable: bool,
    pub success_action: Option<SuccessAction>,
}

#[derive(Clone, Debug)]
pub enum SuccessAction {
    Url(url::Url, String),
    Message(String),
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a: de::CallbackResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        let success_action = a
            .success_action
            .and_then(|sa| match sa.get("tag")? as &str {
                "message" => Some(SuccessAction::Message(sa.get("message")?.to_owned())),
                "url" => {
                    let url = url::Url::parse(sa.get("url")?).ok()?;
                    Some(SuccessAction::Url(url, sa.get("description")?.to_owned()))
                }
                _ => None,
            });

        Ok(Self {
            pr: a.pr,
            disposable: a.disposable.unwrap_or(true),
            success_action,
        })
    }
}

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let success_action = self.success_action.as_ref().map(|sa| {
            let mut map = std::collections::BTreeMap::new();

            match sa {
                SuccessAction::Message(m) => {
                    map.insert("tag", "message".into());
                    map.insert("message", m.into());
                }
                SuccessAction::Url(u, d) => {
                    map.insert("tag", "url".into());
                    map.insert("description", d.into());
                    map.insert("url", u.to_string().into());
                }
            }

            map
        });

        let cr = ser::CallbackResponse {
            success_action,
            disposable: self.disposable,
            pr: &self.pr,
        };

        f.write_str(&miniserde::json::to_string(&cr))
    }
}

mod ser {
    use crate::serde::Url;
    use miniserde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    pub(super) struct Query<'a> {
        pub tag: &'static str,
        pub metadata: String,
        pub callback: Url<'a>,
        #[serde(rename = "minSendable")]
        pub min_sendable: u64,
        #[serde(rename = "maxSendable")]
        pub max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        pub comment_allowed: u64,
    }

    #[derive(Serialize)]
    pub(super) struct CallbackResponse<'a> {
        pub pr: &'a str,
        pub disposable: bool,
        #[serde(rename = "successAction")]
        pub success_action: Option<BTreeMap<&'static str, std::borrow::Cow<'a, str>>>,
    }
}

mod de {
    use crate::serde::Url;
    use miniserde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    pub(super) struct Query {
        pub metadata: String,
        pub callback: Url<'static>,
        #[serde(rename = "minSendable")]
        pub min_sendable: u64,
        #[serde(rename = "maxSendable")]
        pub max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        pub comment_allowed: Option<u64>,
    }

    #[derive(Deserialize)]
    pub(super) struct CallbackResponse {
        pub pr: String,
        pub disposable: Option<bool>,
        #[serde(rename = "successAction")]
        pub success_action: Option<BTreeMap<String, String>>,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn query_parse_base() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.short_description, "boneco do steve magal");
        assert_eq!(parsed.min, 314);
        assert_eq!(parsed.max, 315);

        assert_eq!(parsed.comment_size, 0);
        assert!(parsed.long_description.is_none());
        assert!(parsed.jpeg.is_none());
        assert!(parsed.png.is_none());
        assert!(parsed.identifier.is_none());
        assert!(parsed.email.is_none());
    }

    #[test]
    fn query_parse_comment_size() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
                "commentAllowed": 140,
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");
        assert_eq!(parsed.comment_size, 140);
    }

    #[test]
    fn query_parse_long_description() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/long-desc\", \"mochila a jato brutal incluida\"]]",
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");
        assert_eq!(
            parsed.long_description.unwrap(),
            "mochila a jato brutal incluida"
        );
    }

    #[test]
    fn query_parse_images() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"image/png;base64\", \"Zm90b2JydXRhbA==\"],[\"image/jpeg;base64\", \"aW1hZ2VtYnJ1dGFs\"]]",
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");
        assert_eq!(parsed.jpeg.unwrap(), b"imagembrutal");
        assert_eq!(parsed.png.unwrap(), b"fotobrutal");
    }

    #[test]
    fn query_parse_identifier() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/identifier\", \"steve@magal.brutal\"]]",
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");
        assert_eq!(parsed.identifier.unwrap(), "steve@magal.brutal");
    }

    #[test]
    fn query_parse_email() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/email\", \"steve@magal.brutal\"]]",
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");
        assert_eq!(parsed.email.unwrap(), "steve@magal.brutal");
    }

    #[test]
    fn query_render_base() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: 0,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn query_render_comment_size() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: 140,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":140}"#
        );
    }

    #[test]
    fn query_render_long_description() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: Some(String::from("mochila a jato brutal incluida")),
            jpeg: None,
            png: None,
            comment_size: 0,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"text/long-desc\",\"mochila a jato brutal incluida\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn query_render_images() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: 0,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn query_render_identifier() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: 0,
            min: 314,
            max: 315,
            identifier: Some(String::from("steve@magal.brutal")),
            email: None,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"],[\"text/identifier\",\"steve@magal.brutal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn query_render_email() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: 0,
            min: 314,
            max: 315,
            identifier: None,
            email: Some(String::from("steve@magal.brutal")),
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"],[\"text/email\",\"steve@magal.brutal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn callback() {
        let input = r#"
            {
                "callback": "https://yuri?o=callback",
                "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
                "maxSendable": 315,
                "minSendable": 314
            }
        "#;

        let parsed = input.parse::<super::Query>().expect("parse");

        assert_eq!(
            parsed.clone().callback("comentario", 314).to_string(),
            "https://yuri/?o=callback&comment=comentario&amount=314"
        );

        assert_eq!(
            parsed.callback("", 314).to_string(),
            "https://yuri/?o=callback&amount=314"
        );
    }

    #[test]
    fn callback_response_parse_base() {
        let input = r#"
            { "pr": "pierre" }
        "#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(parsed.success_action.is_none());
        assert_eq!(parsed.pr, "pierre");
        assert!(parsed.disposable);
    }

    #[test]
    fn callback_response_parse_disposable() {
        let input = r#"
            { "pr": "", "disposable": true }
        "#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(parsed.disposable);

        let input = r#"
            { "pr": "", "disposable": false }
        "#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(!parsed.disposable);
    }

    #[test]
    fn callback_response_parse_success_actions() {
        let input = r#"
            { "pr": "", "successAction": { "tag": "message", "message": "obrigado!" } }
        "#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");

        let Some(super::SuccessAction::Message(m)) = parsed.success_action else {
            panic!("bad success action");
        };

        assert_eq!(m, "obrigado!");

        let input = r#"
            { "pr": "", "successAction": { "tag": "url", "description": "valeu demais", "url": "http://eh.nois" } }
        "#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");

        let Some(super::SuccessAction::Url(u, d)) = parsed.success_action else {
            panic!("bad success action");
        };

        assert_eq!(u.to_string(), "http://eh.nois/");
        assert_eq!(d, "valeu demais");
    }
}