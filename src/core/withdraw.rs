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
    #[must_use]
    pub fn callback(self, pr: String) -> CallbackRequest {
        CallbackRequest {
            url: self.callback,
            k1: self.k1,
            pr,
        }
    }
}

pub struct CallbackRequest {
    pub url: url::Url,
    pub k1: String,
    pub pr: String,
}

impl CallbackRequest {
    #[must_use]
    pub fn url(mut self) -> url::Url {
        let query = [("k1", self.k1), ("pr", self.pr)];
        self.url.query_pairs_mut().extend_pairs(query);
        self.url
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
    fn query_parse() {
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
    fn query_render() {
        let query = super::Query {
            k1: String::from("caum"),
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            description: String::from("verde com bolinhas"),
            min: 314,
            max: 315,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"withdrawRequest","k1":"caum","callback":"https://yuri/?o=callback","defaultDescription":"verde com bolinhas","minWithdrawable":314,"maxWithdrawable":315}"#
        );
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
            parsed.callback(String::from("pierre")).url().to_string(),
            "https://yuri/?o=callback&k1=caum&pr=pierre"
        );
    }

    #[test]
    fn callback_response_parse() {
        assert!(matches!(
            r#"{ "status": "OK"}"#.parse().unwrap(),
            super::CallbackResponse::Ok
        ));

        assert!(matches!(
            r#"{ "status": "ERROR", "reason": "razao" }"#.parse().unwrap(),
            super::CallbackResponse::Error(r) if r == "razao"
        ));
    }

    #[test]
    fn callback_response_render() {
        assert_eq!(
            super::CallbackResponse::Ok.to_string(),
            r#"{"status":"OK"}"#
        );
        assert_eq!(
            super::CallbackResponse::Error(String::from("razao")).to_string(),
            r#"{"reason":"razao","status":"ERROR"}"#
        );
    }
}
