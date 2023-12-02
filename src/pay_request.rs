pub const TAG: &str = "payRequest";

#[derive(Clone, Debug)]
pub struct PayRequest<'a> {
    client: &'a reqwest::Client,
    callback: crate::serde::Url,
    pub short_description: String,
    pub long_description: Option<String>,
    pub jpeg: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
    pub min: u64,
    pub max: u64,
}

pub(crate) fn build<'a>(
    s: &str,
    client: &'a reqwest::Client,
) -> Result<PayRequest<'a>, &'static str> {
    use base64::{prelude::BASE64_STANDARD, Engine};
    use miniserde::{json::Value, Deserialize};

    #[derive(Deserialize)]
    struct Deserialized {
        metadata: String,
        callback: crate::serde::Url,
        #[serde(rename = "minSendable")]
        min_sendable: u64,
        #[serde(rename = "maxSendable")]
        max_sendable: u64,
    }

    let d: Deserialized = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;
    let metadata = miniserde::json::from_str::<Vec<(String, Value)>>(&d.metadata)
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
        callback: d.callback,
        min: d.min_sendable,
        max: d.max_sendable,
        short_description,
        long_description,
        jpeg,
        png,
    })
}

pub struct PayRequestResponse {
    pub pr: String,
    pub disposable: bool,
}

impl PayRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn generate_invoice(
        mut self,
        millisatoshis: u64,
    ) -> Result<PayRequestResponse, &'static str> {
        #[derive(miniserde::Deserialize)]
        struct Deserialized {
            pr: String,
            disposable: Option<bool>,
        }

        self.callback
            .0
            .query_pairs_mut()
            .append_pair("amount", &millisatoshis.to_string());

        let response = self
            .client
            .get(self.callback.0)
            .send()
            .await
            .map_err(|_| "request failed")?;

        let body = response.text().await.map_err(|_| "body failed")?;
        let response =
            miniserde::json::from_str::<Deserialized>(&body).map_err(|_| "deserialize failed")?;

        Ok(PayRequestResponse {
            pr: response.pr,
            disposable: response.disposable.unwrap_or(false),
        })
    }
}
