pub const TAG: &str = "withdrawalRequest";

#[derive(Debug, miniserde::Deserialize)]
pub struct WithdrawalRequest {
    callback: crate::serde::Url,
    k1: String,
    #[serde(rename = "defaultDescription")]
    pub description: String,
    #[serde(rename = "minWithdrawable")]
    pub min: u64,
    #[serde(rename = "maxWithdrawable")]
    pub max: u64,
}

impl WithdrawalRequest {
    pub fn callback(mut self, pr: &str) -> url::Url {
        self.callback
            .0
            .query_pairs_mut()
            .extend_pairs([("k1", &self.k1 as &str), ("pr", pr)]);

        self.callback.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let input = r#"
			{
			    "callback": "https://bipa.app/callback?q=1",
			    "k1": "caum",
			    "defaultDescription": "descrição",
			    "minWithdrawable": 1000,
			    "maxWithdrawable": 2000
			}
        "#;

        let cr = miniserde::json::from_str::<super::WithdrawalRequest>(input).expect("parse");

        assert_eq!(cr.callback.0.to_string(), "https://bipa.app/callback?q=1");
        assert_eq!(cr.description, "descrição");
        assert_eq!(cr.min, 1000);
        assert_eq!(cr.max, 2000);
        assert_eq!(cr.k1, "caum");

        assert_eq!(
            cr.callback("peerre").to_string(),
            "https://bipa.app/callback?q=1&k1=caum&pr=peerre"
        );
    }
}
