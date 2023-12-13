#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub k1: String,
    pub callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl TryFrom<&[u8]> for Entrypoint {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let d: de::Entrypoint = serde_json::from_slice(s).map_err(|_| "deserialize failed")?;

        Ok(Entrypoint {
            k1: d.k1,
            callback: d.callback,
            description: d.default_description,
            min: d.min_withdrawable,
            max: d.max_withdrawable,
        })
    }
}

impl std::str::FromStr for Entrypoint {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let d: de::Entrypoint = serde_urlencoded::from_str(s).map_err(|_| "deserialize failed")?;

        Ok(Entrypoint {
            k1: d.k1,
            callback: d.callback,
            description: d.default_description,
            min: d.min_withdrawable,
            max: d.max_withdrawable,
        })
    }
}

impl Entrypoint {
    #[must_use]
    pub fn submit<'a>(&'a self, pr: &'a str) -> Callback {
        Callback {
            url: &self.callback,
            k1: &self.k1,
            pr,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Callback<'a> {
    pub url: &'a url::Url,
    pub k1: &'a str,
    pub pr: &'a str,
}

impl std::fmt::Display for Callback<'_> {
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
    pub(super) struct Entrypoint {
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
    fn entrypoint_bytes_parse() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "defaultDescription": "verde com bolinhas",
            "minWithdrawable": 314,
            "maxWithdrawable": 315,
            "k1": "caum"
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.description, "verde com bolinhas");
        assert_eq!(parsed.k1, "caum");
        assert_eq!(parsed.max, 315);
        assert_eq!(parsed.min, 314);
    }

    #[test]
    fn entrypoint_string_parse() {
        let input = "lnurlw://there.is/no\
            ?s=poon\
            &tag=withdrawRequest\
            &k1=caum\
            &minWithdrawable=314\
            &maxWithdrawable=315\
            &defaultDescription=descricao\
            &callback=https://call.back";

        let parsed: super::Entrypoint = input.parse().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://call.back/");
        assert_eq!(parsed.description, "descricao");
        assert_eq!(parsed.k1, "caum");
        assert_eq!(parsed.min, 314);
        assert_eq!(parsed.max, 315);
    }

    #[test]
    fn callback_render() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "defaultDescription": "verde com bolinhas",
            "minWithdrawable": 314,
            "maxWithdrawable": 315,
            "k1": "caum"
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(
            parsed.submit("pierre").to_string(),
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
