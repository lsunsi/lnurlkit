pub struct Query {
    pub k1: String,
    pub callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = serde_json::to_string(&ser::Query {
            tag: super::TAG,
            callback: &self.callback,
            default_description: &self.description,
            min_withdrawable: self.min,
            max_withdrawable: self.max,
            k1: &self.k1,
        });
        f.write_str(&ser.map_err(|_| std::fmt::Error)?)
    }
}

pub struct CallbackRequest {
    pub k1: String,
    pub pr: String,
}

impl std::str::FromStr for CallbackRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let qs = s
            .split('&')
            .filter_map(|s| s.split_once('='))
            .collect::<std::collections::HashMap<_, _>>();

        let k1 = String::from(*qs.get("k1").ok_or("missing k1")?);
        let pr = String::from(*qs.get("pr").ok_or("missing pr")?);

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

        let ser = serde_json::to_string(&map).map_err(|_| std::fmt::Error)?;
        f.write_str(&ser)
    }
}

mod ser {
    use serde::Serialize;
    use url::Url;

    #[derive(Serialize)]
    pub(super) struct Query<'a> {
        pub tag: &'static str,
        pub k1: &'a str,
        pub callback: &'a Url,
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

        assert_eq!(parsed.pr, "pierre");
        assert_eq!(parsed.k1, "caum");
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
