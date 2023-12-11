#[derive(Clone, Debug)]
pub struct Response {
    pub callback: url::Url,
    pub metadata_raw: String,
    pub short_description: String,
    pub long_description: Option<String>,
    pub identifier: Option<String>,
    pub email: Option<String>,
    pub jpeg: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
    pub comment_size: Option<u64>,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for Response {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use base64::{prelude::BASE64_STANDARD, Engine};
        use serde_json::Value;

        let p: de::Response = serde_json::from_str(s).map_err(|_| "deserialize failed")?;

        let metadata = serde_json::from_str::<Vec<(String, Value)>>(&p.metadata)
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

        Ok(Response {
            metadata_raw: p.metadata,
            callback: p.callback,
            comment_size: p.comment_allowed,
            min: p.min_sendable,
            max: p.max_sendable,
            short_description,
            long_description,
            identifier,
            email,
            jpeg,
            png,
        })
    }
}

impl Response {
    #[must_use]
    pub fn callback<'a>(
        &'a self,
        millisatoshis: u64,
        comment: Option<&'a str>,
    ) -> CallbackQuery<'a> {
        CallbackQuery {
            url: &self.callback,
            millisatoshis,
            comment,
        }
    }
}

pub struct CallbackQuery<'a> {
    pub url: &'a url::Url,
    pub comment: Option<&'a str>,
    pub millisatoshis: u64,
}

impl std::fmt::Display for CallbackQuery<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let query = super::serde::CallbackQuery {
            comment: self.comment,
            amount: self.millisatoshis,
        };

        let querystr = serde_urlencoded::to_string(query).map_err(|_| std::fmt::Error)?;
        let sep = if self.url.query().is_some() { '&' } else { '?' };
        write!(f, "{}{sep}{querystr}", self.url)
    }
}

#[derive(Clone, Debug)]
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
        let a: de::CallbackResponse = serde_json::from_str(s).map_err(|_| "deserialize failed")?;

        let success_action = a
            .success_action
            .and_then(|sa| match sa.get("tag")? as &str {
                "message" => Some(SuccessAction::Message(String::from(sa.get("message")?))),
                "url" => {
                    let url = url::Url::parse(sa.get("url")?).ok()?;
                    let description = String::from(sa.get("description")?);
                    Some(SuccessAction::Url(url, description))
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

mod de {
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use url::Url;

    #[derive(Deserialize)]
    pub(super) struct Response {
        pub metadata: String,
        pub callback: Url,
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
    fn response_parse_base() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.short_description, "boneco do steve magal");
        assert_eq!(
            parsed.metadata_raw,
            "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]"
        );
        assert_eq!(parsed.min, 314);
        assert_eq!(parsed.max, 315);

        assert!(parsed.comment_size.is_none());
        assert!(parsed.long_description.is_none());
        assert!(parsed.jpeg.is_none());
        assert!(parsed.png.is_none());
        assert!(parsed.identifier.is_none());
        assert!(parsed.email.is_none());
    }

    #[test]
    fn response_parse_comment_size() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "commentAllowed": 140,
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");
        assert_eq!(parsed.comment_size.unwrap(), 140);
    }

    #[test]
    fn response_parse_long_description() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/long-desc\", \"mochila a jato brutal incluida\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");
        assert_eq!(
            parsed.long_description.unwrap(),
            "mochila a jato brutal incluida"
        );
    }

    #[test]
    fn response_parse_images() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"image/png;base64\", \"Zm90b2JydXRhbA==\"],[\"image/jpeg;base64\", \"aW1hZ2VtYnJ1dGFs\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");
        assert_eq!(parsed.jpeg.unwrap(), b"imagembrutal");
        assert_eq!(parsed.png.unwrap(), b"fotobrutal");
    }

    #[test]
    fn response_parse_identifier() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/identifier\", \"steve@magal.brutal\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");
        assert_eq!(parsed.identifier.unwrap(), "steve@magal.brutal");
    }

    #[test]
    fn response_parse_email() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/email\", \"steve@magal.brutal\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");
        assert_eq!(parsed.email.unwrap(), "steve@magal.brutal");
    }

    #[test]
    fn callback_query_render_base() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");

        assert_eq!(
            parsed.callback(314, None).to_string(),
            "https://yuri/?o=callback&amount=314"
        );
    }

    #[test]
    fn callback_query_render_comment() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");

        assert_eq!(
            parsed.callback(314, Some("comentario")).to_string(),
            "https://yuri/?o=callback&comment=comentario&amount=314"
        );
    }

    #[test]
    fn callback_response_parse_base() {
        let input = r#"{ "pr": "pierre" }"#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(parsed.success_action.is_none());
        assert_eq!(parsed.pr, "pierre");
        assert!(parsed.disposable);
    }

    #[test]
    fn callback_response_parse_disposable() {
        let input = r#"{ "pr": "", "disposable": true }"#;
        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(parsed.disposable);

        let input = r#"{ "pr": "", "disposable": false }"#;
        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(!parsed.disposable);
    }

    #[test]
    fn callback_response_parse_success_actions() {
        let input =
            r#"{ "pr": "", "successAction": { "tag": "message", "message": "obrigado!" } }"#;

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
