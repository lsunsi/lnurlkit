use std::collections::BTreeMap;

pub const TAG: &str = "payRequest";

#[derive(Clone, Debug)]
pub struct PayRequest<'a> {
    client: &'a reqwest::Client,
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

pub(crate) fn build<'a>(
    s: &str,
    client: &'a reqwest::Client,
) -> Result<PayRequest<'a>, &'static str> {
    use base64::{prelude::BASE64_STANDARD, Engine};
    use miniserde::{json::Value, Deserialize};

    #[derive(Deserialize)]
    struct Payload {
        metadata: String,
        callback: crate::serde::Url,
        #[serde(rename = "minSendable")]
        min_sendable: u64,
        #[serde(rename = "maxSendable")]
        max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        comment_allowed: Option<u64>,
        #[serde(rename = "successAction")]
        success_action: Option<BTreeMap<String, String>>,
    }

    let p: Payload = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;
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
        client,
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

#[derive(Debug)]
pub struct GeneratedInvoice {
    pub pr: String,
    pub disposable: bool,
}

impl PayRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn generate_invoice(
        mut self,
        comment: &str,
        millisatoshis: u64,
    ) -> Result<GeneratedInvoice, &'static str> {
        #[derive(miniserde::Deserialize)]
        struct Deserialized {
            pr: String,
            disposable: Option<bool>,
        }

        self.callback.0.query_pairs_mut().extend_pairs(
            [
                (!comment.is_empty()).then_some(("comment", comment)),
                Some(("amount", &millisatoshis.to_string())),
            ]
            .into_iter()
            .flatten(),
        );

        let response = self
            .client
            .get(self.callback.0)
            .send()
            .await
            .map_err(|_| "request failed")?;

        let body = response.text().await.map_err(|_| "body failed")?;
        let response =
            miniserde::json::from_str::<Deserialized>(&body).map_err(|_| "deserialize failed")?;

        Ok(GeneratedInvoice {
            pr: response.pr,
            disposable: response.disposable.unwrap_or(true),
        })
    }
}
