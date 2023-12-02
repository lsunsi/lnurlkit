pub const TAG: &str = "payRequest";

#[derive(Debug, Clone)]
pub struct PayRequest {
    callback: crate::serde::Url,
    pub short_description: String,
    pub long_description: Option<String>,
    pub jpeg: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
    pub min: u64,
    pub max: u64,
}

impl std::str::FromStr for PayRequest {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use base64::{prelude::BASE64_STANDARD, Engine};
        use miniserde::{json::Value, Deserialize};

        #[derive(Deserialize)]
        struct Deserialized {
            metadata: String,
            callback: crate::serde::Url,
            #[serde(rename = "minSendable")]
            min_sendable: u64,
            #[serde(rename = "maxSendable")]
            max_sendable: u64,
        }

        let d: Deserialized = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;
        let metadata = miniserde::json::from_str::<Vec<(String, Value)>>(&d.metadata)
            .map_err(|_| "deserialize metadata failed")?;

        let short_description = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/plain").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            })
            .ok_or("short description failed")?;

        let long_description = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/long-desc").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            });

        let jpeg = metadata
            .iter()
            .find_map(|(k, v)| (k == "image/jpeg;base64").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => BASE64_STANDARD.decode(s).ok(),
                _ => None,
            });

        let png = metadata
            .iter()
            .find_map(|(k, v)| (k == "image/png;base64").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => BASE64_STANDARD.decode(s).ok(),
                _ => None,
            });

        Ok(PayRequest {
            callback: d.callback,
            min: d.min_sendable,
            max: d.max_sendable,
            short_description,
            long_description,
            jpeg,
            png,
        })
    }
}

impl PayRequest {
    pub fn callback(mut self, millisatoshis: u64) -> url::Url {
        self.callback
            .0
            .query_pairs_mut()
            .append_pair("amount", &millisatoshis.to_string());

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
                "maxSendable": 200000,
                "minSendable": 100000,
                "metadata": "[[\"text/plain\", \"olá\"]]"
            }
        "#;

        let cr = input.parse::<super::PayRequest>().expect("parse");

        assert_eq!(cr.callback.0.to_string(), "https://bipa.app/callback?q=1");
        assert_eq!(cr.max, 200000);
        assert_eq!(cr.min, 100000);
        assert_eq!(cr.short_description, "olá");

        assert!(cr.long_description.is_none());
        assert!(cr.jpeg.is_none());
        assert!(cr.png.is_none());

        assert_eq!(
            cr.callback(123).to_string(),
            "https://bipa.app/callback?q=1&amount=123"
        );

        let input = r#"
            {
                "callback": "https://bipa.app/callback?q=1",
                "maxSendable": 200000,
                "minSendable": 100000,
                "metadata": "[[\"text/plain\", \"olá\"],[\"text/long-desc\", \"oie\"],[\"image/png;base64\", \"YWJj\"],[\"image/jpeg;base64\", \"cXdlcnR5\"]]"
            }
        "#;

        let cr = input.parse::<super::PayRequest>().expect("parse");

        assert_eq!(cr.callback.0.to_string(), "https://bipa.app/callback?q=1");
        assert_eq!(cr.max, 200000);
        assert_eq!(cr.min, 100000);
        assert_eq!(cr.short_description, "olá");
        assert_eq!(cr.long_description.unwrap(), "oie");
        assert_eq!(cr.jpeg.unwrap(), b"qwerty");
        assert_eq!(cr.png.unwrap(), b"abc");
    }
}
