pub const TAG: &str = "payRequest";

#[derive(Clone, Debug)]
pub struct PayRequest {
    callback: crate::serde::Url,
    pub short_description: String,
    pub long_description: Option<String>,
    pub success_action: Option<SuccessAction>,
    pub jpeg: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
    pub comment_size: u64,
    pub min: u64,
    pub max: u64,
}

#[derive(Clone, Debug)]
pub enum SuccessAction {
    Url(url::Url, String),
    Message(String),
}

impl std::str::FromStr for PayRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use base64::{prelude::BASE64_STANDARD, Engine};
        use miniserde::json::Value;

        let p: de::QueryResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;
        let comment_size = p.comment_allowed.unwrap_or(0);

        let success_action = p
            .success_action
            .and_then(|sa| match sa.get("tag")? as &str {
                "message" => Some(SuccessAction::Message(sa.get("message")?.to_owned())),
                "url" => {
                    let url = url::Url::parse(sa.get("url")?).ok()?;
                    Some(SuccessAction::Url(url, sa.get("description")?.to_owned()))
                }
                _ => None,
            });

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

        Ok(PayRequest {
            callback: p.callback,
            min: p.min_sendable,
            max: p.max_sendable,
            short_description,
            long_description,
            success_action,
            comment_size,
            jpeg,
            png,
        })
    }
}

impl std::fmt::Display for PayRequest {
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
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        );

        let success_action = self.success_action.as_ref().map(|sa| {
            let mut map = std::collections::BTreeMap::new();

            match sa {
                SuccessAction::Message(m) => {
                    map.insert("message", m.into());
                }
                SuccessAction::Url(u, d) => {
                    map.insert("description", d.into());
                    map.insert("url", u.to_string().into());
                }
            }

            map
        });

        f.write_str(&miniserde::json::to_string(&ser::QueryResponse {
            metadata,
            callback: &self.callback,
            min_sendable: self.min,
            max_sendable: self.max,
            comment_allowed: self.comment_size,
            success_action,
        }))
    }
}

impl PayRequest {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback(mut self, comment: &str, millisatoshis: u64) -> url::Url {
        self.callback.0.query_pairs_mut().extend_pairs(
            [
                (!comment.is_empty()).then_some(("comment", comment)),
                Some(("amount", &millisatoshis.to_string())),
            ]
            .into_iter()
            .flatten(),
        );

        self.callback.0
    }
}

#[derive(Debug)]
pub struct CallbackResponse {
    pub pr: String,
    pub disposable: bool,
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a: de::CallbackResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(Self {
            pr: a.pr,
            disposable: a.disposable.unwrap_or(true),
        })
    }
}

mod ser {
    use crate::serde::Url;
    use miniserde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    pub(super) struct QueryResponse<'a> {
        pub metadata: String,
        pub callback: &'a Url,
        #[serde(rename = "minSendable")]
        pub min_sendable: u64,
        #[serde(rename = "maxSendable")]
        pub max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        pub comment_allowed: u64,
        #[serde(rename = "successAction")]
        pub success_action: Option<BTreeMap<&'static str, std::borrow::Cow<'a, str>>>,
    }
}

mod de {
    use crate::serde::Url;
    use miniserde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize)]
    pub(super) struct QueryResponse {
        pub metadata: String,
        pub callback: Url,
        #[serde(rename = "minSendable")]
        pub min_sendable: u64,
        #[serde(rename = "maxSendable")]
        pub max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        pub comment_allowed: Option<u64>,
        #[serde(rename = "successAction")]
        pub success_action: Option<BTreeMap<String, String>>,
    }

    #[derive(Deserialize)]
    pub(super) struct CallbackResponse {
        pub pr: String,
        pub disposable: Option<bool>,
    }
}
