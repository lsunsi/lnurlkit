#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub k1: String,
    pub callback: url::Url,
    pub description: String,
    pub min: u64,
    pub max: u64,
}

impl TryFrom<Entrypoint> for Vec<u8> {
    type Error = &'static str;

    fn try_from(r: Entrypoint) -> Result<Self, Self::Error> {
        serde_json::to_vec(&ser::Entrypoint {
            tag: super::TAG,
            callback: &r.callback,
            default_description: &r.description,
            min_withdrawable: r.min,
            max_withdrawable: r.max,
            k1: &r.k1,
        })
        .map_err(|_| "serialize failed")
    }
}

impl std::fmt::Display for Entrypoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_urlencoded::to_string(ser::Entrypoint {
            tag: super::TAG,
            callback: &self.callback,
            default_description: &self.description,
            min_withdrawable: self.min,
            max_withdrawable: self.max,
            k1: &self.k1,
        })
        .map_err(|_| std::fmt::Error)?;
        f.write_str(&s)
    }
}

pub struct Callback {
    pub k1: String,
    pub pr: String,
}

impl<'a> TryFrom<&'a str> for Callback {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        serde_urlencoded::from_str::<super::serde::CallbackQuery>(s)
            .map_err(|_| "deserialize failed")
            .map(|query| Callback {
                k1: String::from(query.k1),
                pr: String::from(query.pr),
            })
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
    pub(super) struct Entrypoint<'a> {
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
    fn entrypoint_render() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            description: String::from("verde com bolinhas"),
            k1: String::from("caum"),
            min: 314,
            max: 315,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"withdrawRequest","k1":"caum","callback":"https://yuri/?o=callback","defaultDescription":"verde com bolinhas","minWithdrawable":314,"maxWithdrawable":315}"#
        );
    }

    #[test]
    fn callback_parse() {
        let input = "k1=caum&pr=pierre";
        let parsed: super::Callback = input.try_into().expect("parse");

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
