pub const TAG: &str = "withdrawalRequest";

#[derive(Clone, Debug)]
pub struct WithdrawalRequest<'a> {
    client: &'a reqwest::Client,
    callback: crate::serde::Url,
    k1: String,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

pub(crate) fn build<'a>(
    s: &str,
    client: &'a reqwest::Client,
) -> Result<WithdrawalRequest<'a>, &'static str> {
    #[derive(miniserde::Deserialize)]
    struct Deserialized {
        k1: String,
        callback: crate::serde::Url,
        #[serde(rename = "defaultDescription")]
        default_description: String,
        #[serde(rename = "minWithdrawable")]
        min_withdrawable: u64,
        #[serde(rename = "maxWithdrawable")]
        max_withdrawable: u64,
    }

    let d: Deserialized = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

    Ok(WithdrawalRequest {
        client,
        k1: d.k1,
        callback: d.callback,
        description: d.default_description,
        min: d.min_withdrawable,
        max: d.max_withdrawable,
    })
}

impl WithdrawalRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(mut self, pr: &str) -> Result<(), &'static str> {
        self.callback
            .0
            .query_pairs_mut()
            .extend_pairs([("k1", &self.k1 as &str), ("pr", pr)]);

        self.client
            .get(self.callback.0)
            .send()
            .await
            .map_err(|_| "request failed")?;

        Ok(())
    }
}
