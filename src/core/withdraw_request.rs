pub const TAG: &str = "withdrawRequest";

#[derive(Clone, Debug)]
pub struct WithdrawRequest {
    k1: String,
    callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for WithdrawRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: serde::QueryResponse =
            miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(WithdrawRequest {
            k1: d.k1,
            callback: d.callback.0,
            description: d.default_description,
            min: d.min_withdrawable,
            max: d.max_withdrawable,
        })
    }
}

impl WithdrawRequest {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    #[must_use]
    pub fn callback(mut self, pr: &str) -> url::Url {
        self.callback
            .query_pairs_mut()
            .extend_pairs([("k1", &self.k1 as &str), ("pr", pr)]);

        self.callback
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

#[cfg(test)]
mod tests {
    #[test]
    fn parse() {
        let input = r#"
            {
                "k1": "caum",
                "callback": "https://yuri?o=callback",
                "defaultDescription": "verde com bolinhas",
                "minWithdrawable": 314,
                "maxWithdrawable": 315
            }
        "#;

        let parsed = input.parse::<super::WithdrawRequest>().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.description, "verde com bolinhas");
        assert_eq!(parsed.k1, "caum");
        assert_eq!(parsed.max, 315);
        assert_eq!(parsed.min, 314);
    }

    #[test]
    fn callback() {
        let input = r#"
            {
                "k1": "caum",
                "callback": "https://yuri?o=callback",
                "defaultDescription": "verde com bolinhas",
                "minWithdrawable": 314,
                "maxWithdrawable": 315
            }
        "#;

        let parsed = input.parse::<super::WithdrawRequest>().expect("parse");

        assert_eq!(
            parsed.callback("pierre").to_string(),
            "https://yuri/?o=callback&k1=caum&pr=pierre"
        );
    }
}
