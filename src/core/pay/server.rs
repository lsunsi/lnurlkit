#[derive(Clone, Debug)]
pub struct Response {
    pub callback: url::Url,
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

impl TryFrom<Response> for Vec<u8> {
    type Error = &'static str;

    fn try_from(r: Response) -> Result<Self, Self::Error> {
        use base64::{prelude::BASE64_STANDARD, Engine};

        let metadata = serde_json::to_string(
            &[
                Some(("text/plain", r.short_description.clone())),
                r.long_description
                    .as_ref()
                    .map(|s| ("text/long-desc", s.clone())),
                r.jpeg
                    .as_ref()
                    .map(|s| ("image/jpeg;base64", BASE64_STANDARD.encode(s))),
                r.png
                    .as_ref()
                    .map(|s| ("image/png;base64", BASE64_STANDARD.encode(s))),
                r.identifier
                    .as_ref()
                    .map(|s| ("text/identifier", s.clone())),
                r.email.as_ref().map(|s| ("text/email", s.clone())),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )
        .map_err(|_| "serialize failed")?;

        serde_json::to_vec(&ser::Response {
            tag: super::TAG,
            metadata,
            callback: &r.callback,
            min_sendable: r.min,
            max_sendable: r.max,
            comment_allowed: r.comment_size.unwrap_or(0),
        })
        .map_err(|_| "serialize failed")
    }
}

pub struct CallbackQuery {
    pub millisatoshis: u64,
    pub comment: Option<String>,
}

impl<'a> TryFrom<&'a str> for CallbackQuery {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        serde_urlencoded::from_str::<super::serde::CallbackQuery>(s)
            .map_err(|_| "deserialize failed")
            .map(|query| CallbackQuery {
                millisatoshis: query.amount,
                comment: query.comment.map(String::from),
            })
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

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let success_action = self.success_action.as_ref().map(|sa| {
            let mut map = std::collections::BTreeMap::new();

            match sa {
                SuccessAction::Message(m) => {
                    map.insert("tag", "message");
                    map.insert("message", m);
                }
                SuccessAction::Url(u, d) => {
                    map.insert("tag", "url");
                    map.insert("description", d);
                    map.insert("url", u.as_str());
                }
            }

            map
        });

        let cr = ser::CallbackResponse {
            success_action,
            disposable: self.disposable,
            pr: &self.pr,
        };

        f.write_str(&serde_json::to_string(&cr).map_err(|_| std::fmt::Error)?)
    }
}

mod ser {
    use serde::Serialize;
    use std::collections::BTreeMap;
    use url::Url;

    #[derive(Serialize)]
    pub(super) struct Response<'a> {
        pub tag: &'static str,
        pub metadata: String,
        pub callback: &'a Url,
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
        pub success_action: Option<BTreeMap<&'static str, &'a str>>,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn response_render_base() {
        let query = super::Response {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn response_render_comment_size() {
        let query = super::Response {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: Some(140),
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":140}"#
        );
    }

    #[test]
    fn response_render_long_description() {
        let query = super::Response {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: Some(String::from("mochila a jato brutal incluida")),
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"text/long-desc\",\"mochila a jato brutal incluida\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn response_render_images() {
        let query = super::Response {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn response_render_identifier() {
        let query = super::Response {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: None,
            min: 314,
            max: 315,
            identifier: Some(String::from("steve@magal.brutal")),
            email: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"],[\"text/identifier\",\"steve@magal.brutal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn response_render_email() {
        let query = super::Response {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: Some(String::from("steve@magal.brutal")),
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"],[\"text/email\",\"steve@magal.brutal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn callback_query_parse_base() {
        let input = "amount=314";
        let parsed: super::CallbackQuery = input.try_into().expect("parse");

        assert_eq!(parsed.millisatoshis, 314);
        assert!(parsed.comment.is_none());
    }

    #[test]
    fn callback_query_parse_comment() {
        let input = "amount=314&comment=comentario";
        let parsed: super::CallbackQuery = input.try_into().expect("parse");

        assert_eq!(parsed.millisatoshis, 314);
        assert_eq!(parsed.comment.unwrap(), "comentario");
    }

    #[test]
    fn callback_response_render_base() {
        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: None,
            disposable: true,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":true,"successAction":null}"#
        );
    }

    #[test]
    fn callback_response_render_disposable() {
        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: None,
            disposable: false,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":false,"successAction":null}"#
        );
    }

    #[test]
    fn callback_response_render_success_actions() {
        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: Some(super::SuccessAction::Message(String::from("obrigado!"))),
            disposable: false,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":false,"successAction":{"message":"obrigado!","tag":"message"}}"#
        );

        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: Some(super::SuccessAction::Url(
                url::Url::parse("http://recibo").expect("url"),
                String::from("segue recibo"),
            )),
            disposable: false,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":false,"successAction":{"description":"segue recibo","tag":"url","url":"http://recibo/"}}"#
        );
    }
}
