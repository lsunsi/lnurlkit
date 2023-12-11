#[derive(Clone, Debug)]
pub struct Response {
    k1: String,
    callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for Response {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: de::Response = serde_json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(Response {
            k1: d.k1,
            callback: d.callback,
            description: d.default_description,
            min: d.min_withdrawable,
            max: d.max_withdrawable,
        })
    }
}

impl Response {
    #[must_use]
    pub fn callback<'a>(&'a self, pr: &'a str) -> CallbackQuery {
        CallbackQuery {
            url: &self.callback,
            k1: &self.k1,
            pr,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallbackQuery<'a> {
    pub url: &'a url::Url,
    pub k1: &'a str,
    pub pr: &'a str,
}

impl std::fmt::Display for CallbackQuery<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let query = super::serde::CallbackQuery {
            k1: self.k1,
            pr: self.pr,
        };

        let querystr = serde_urlencoded::to_string(query).map_err(|_| std::fmt::Error)?;
        let sep = if self.url.query().is_some() { '&' } else { '?' };
        write!(f, "{}{sep}{querystr}", self.url)
    }
}

#[derive(Clone, Debug)]
pub enum CallbackResponse {
    Error { reason: String },
    Ok,
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = serde_json::from_str::<std::collections::BTreeMap<String, String>>(s)
            .map_err(|_| "bad json")?;

        match map.get("status").map(|s| s as &str) {
            Some("OK") => Ok(CallbackResponse::Ok),
            Some("ERROR") => {
                let reason = String::from(map.get("reason").ok_or("error without reason")?);
                Ok(CallbackResponse::Error { reason })
            }
            _ => Err("bad status field"),
        }
    }
}

mod de {
    use serde::Deserialize;
    use url::Url;

    #[derive(Deserialize)]
    pub(super) struct Response {
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
    fn response_parse() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "defaultDescription": "verde com bolinhas",
            "minWithdrawable": 314,
            "maxWithdrawable": 315,
            "k1": "caum"
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.description, "verde com bolinhas");
        assert_eq!(parsed.k1, "caum");
        assert_eq!(parsed.max, 315);
        assert_eq!(parsed.min, 314);
    }

    #[test]
    fn callback_query_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "defaultDescription": "verde com bolinhas",
            "minWithdrawable": 314,
            "maxWithdrawable": 315,
            "k1": "caum"
        }"#;

        let parsed = input.parse::<super::Response>().expect("parse");

        assert_eq!(
            parsed.callback("pierre").to_string(),
            "https://yuri/?o=callback&k1=caum&pr=pierre"
        );
    }

    #[test]
    fn callback_response_parse() {
        assert!(matches!(
            r#"{ "status": "OK" }"#.parse().unwrap(),
            super::CallbackResponse::Ok
        ));

        assert!(matches!(
            r#"{ "status": "ERROR", "reason": "razao" }"#.parse().unwrap(),
            super::CallbackResponse::Error { reason } if reason == "razao"
        ));
    }
}
