pub const TAG: &str = "withdrawRequest";

#[derive(Clone, Debug)]
pub struct Query {
    pub k1: String,
    pub callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: de::Query = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(Query {
            k1: d.k1,
            callback: d.callback.0.into_owned(),
            description: d.default_description,
            min: d.min_withdrawable,
            max: d.max_withdrawable,
        })
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&miniserde::json::to_string(&ser::Query {
            tag: TAG,
            callback: crate::serde::Url(std::borrow::Cow::Borrowed(&self.callback)),
            default_description: &self.description,
            min_withdrawable: self.min,
            max_withdrawable: self.max,
            k1: &self.k1,
        }))
    }
}

impl Query {
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

#[derive(Debug)]
pub enum CallbackResponse {
    Error(String),
    Ok,
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = miniserde::json::from_str::<std::collections::BTreeMap<String, String>>(s)
            .map_err(|_| "bad json")?;

        match map.get("status").map(|s| s as &str) {
            Some("OK") => Ok(CallbackResponse::Ok),
            Some("ERROR") => Ok(CallbackResponse::Error(
                map.get("reason").cloned().unwrap_or_default(),
            )),
            _ => Err("bad status field"),
        }
    }
}

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = std::collections::BTreeMap::new();

        match self {
            CallbackResponse::Error(reason) => {
                map.insert("status", "ERROR");
                map.insert("reason", reason);
            }
            CallbackResponse::Ok => {
                map.insert("status", "OK");
            }
        }

        f.write_str(&miniserde::json::to_string(&map))
    }
}

mod ser {
    use crate::serde::Url;
    use miniserde::Serialize;

    #[derive(Serialize)]
    pub(super) struct Query<'a> {
        pub tag: &'static str,
        pub k1: &'a str,
        pub callback: Url<'a>,
        #[serde(rename = "defaultDescription")]
        pub default_description: &'a str,
        #[serde(rename = "minWithdrawable")]
        pub min_withdrawable: u64,
        #[serde(rename = "maxWithdrawable")]
        pub max_withdrawable: u64,
    }
}

mod de {
    use crate::serde::Url;
    use miniserde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct Query {
        pub k1: String,
        pub callback: Url<'static>,
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

        let parsed = input.parse::<super::Query>().expect("parse");

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

        let parsed = input.parse::<super::Query>().expect("parse");

        assert_eq!(
            parsed.callback("pierre").to_string(),
            "https://yuri/?o=callback&k1=caum&pr=pierre"
        );
    }
}
