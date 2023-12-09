pub struct Query {
    pub k1: String,
    pub callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&miniserde::json::to_string(&ser::Query {
            tag: super::TAG,
            callback: crate::serde::Url(std::borrow::Cow::Borrowed(&self.callback)),
            default_description: &self.description,
            min_withdrawable: self.min,
            max_withdrawable: self.max,
            k1: &self.k1,
        }))
    }
}

pub struct CallbackRequest {
    pub k1: Box<str>,
    pub pr: Box<str>,
}

impl std::str::FromStr for CallbackRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let qs = s
            .split('&')
            .filter_map(|s| s.split_once('='))
            .collect::<std::collections::HashMap<_, _>>();

        let k1 = (*qs.get("k1").ok_or("missing k1")?).into();
        let pr = (*qs.get("pr").ok_or("missing pr")?).into();

        Ok(CallbackRequest { k1, pr })
    }
}

#[derive(Clone, Debug)]
pub enum CallbackResponse {
    Error { reason: String },
    Ok,
}

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = std::collections::BTreeMap::new();

        match self {
            CallbackResponse::Error { reason } => {
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

#[cfg(test)]
mod tests {
    #[test]
    fn query_render() {
        let query = super::Query {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            description: String::from("verde com bolinhas"),
            k1: String::from("caum"),
            min: 314,
            max: 315,
        };

        assert_eq!(
            query.to_string(),
            r#"{"tag":"withdrawRequest","k1":"caum","callback":"https://yuri/?o=callback","defaultDescription":"verde com bolinhas","minWithdrawable":314,"maxWithdrawable":315}"#
        );
    }

    #[test]
    fn callback_request_parse() {
        let input = "k1=caum&pr=pierre";
        let parsed = input.parse::<super::CallbackRequest>().expect("parse");

        assert_eq!(&parsed.pr as &str, "pierre");
        assert_eq!(&parsed.k1 as &str, "caum");
    }

    #[test]
    fn callback_response_render() {
        assert_eq!(
            super::CallbackResponse::Ok.to_string(),
            r#"{"status":"OK"}"#
        );

        assert_eq!(
            super::CallbackResponse::Error {
                reason: String::from("razao")
            }
            .to_string(),
            r#"{"reason":"razao","status":"ERROR"}"#
        );
    }
}
