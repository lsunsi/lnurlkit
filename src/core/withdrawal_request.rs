pub const TAG: &str = "withdrawalRequest";

#[derive(Clone, Debug)]
pub struct WithdrawalRequest {
    k1: String,
    callback: crate::serde::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for WithdrawalRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: serde::QueryResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(WithdrawalRequest {
            k1: d.k1,
            callback: d.callback,
            description: d.default_description,
            min: d.min_withdrawable,
            max: d.max_withdrawable,
        })
    }
}

impl WithdrawalRequest {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback(mut self, pr: &str) -> url::Url {
        self.callback
            .0
            .query_pairs_mut()
            .extend_pairs([("k1", &self.k1 as &str), ("pr", pr)]);

        self.callback.0
    }
}

mod serde {
    use crate::serde::Url;
    use miniserde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct QueryResponse {
        pub k1: String,
        pub callback: Url,
        #[serde(rename = "defaultDescription")]
        pub default_description: String,
        #[serde(rename = "minWithdrawable")]
        pub min_withdrawable: u64,
        #[serde(rename = "maxWithdrawable")]
        pub max_withdrawable: u64,
    }
}
