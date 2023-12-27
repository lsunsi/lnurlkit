#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub callback: url::Url,
    pub metadata_raw: String,
    pub short_description: String,
    pub long_description: Option<String>,
    pub identifier: Option<String>,
    pub email: Option<String>,
    pub jpeg: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
    pub comment_size: Option<u64>,
    pub min: u64,
    pub max: u64,
    pub currencies: Option<Vec<super::Currency>>,
    pub payer: Option<super::PayerRequirements>,
}

#[allow(clippy::too_many_lines)]
impl TryFrom<&[u8]> for Entrypoint {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        use base64::{prelude::BASE64_STANDARD, Engine};
        use serde_json::Value;

        let p: de::Entrypoint = serde_json::from_slice(s).map_err(|_| "deserialize failed")?;

        let currencies = p.currencies.map(|cs| {
            cs.into_iter()
                .map(|c| super::Currency {
                    code: String::from(c.code),
                    name: String::from(c.name),
                    symbol: String::from(c.symbol),
                    decimals: c.decimals,
                    multiplier: c.multiplier,
                    convertible: c.convertible.map(|c| super::CurrencyConvertible {
                        min: c.min,
                        max: c.max,
                    }),
                })
                .collect()
        });

        let payer = p.payer_data.map(|p| super::PayerRequirements {
            name: p.name.map(|p| super::PayerRequirement {
                mandatory: p.mandatory,
            }),
            pubkey: p.pubkey.map(|p| super::PayerRequirement {
                mandatory: p.mandatory,
            }),
            identifier: p.identifier.map(|p| super::PayerRequirement {
                mandatory: p.mandatory,
            }),
            email: p.email.map(|p| super::PayerRequirement {
                mandatory: p.mandatory,
            }),
            auth: p.auth.map(|p| super::PayerRequirementAuth {
                mandatory: p.mandatory,
                k1: p.k1,
            }),
            others: p
                .others
                .into_iter()
                .map(|(k, v)| {
                    (
                        String::from(k),
                        super::PayerRequirement {
                            mandatory: v.mandatory,
                        },
                    )
                })
                .collect(),
        });

        let metadata = serde_json::from_str::<Vec<(String, Value)>>(&p.metadata)
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

        let identifier = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/identifier").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            });

        let email = metadata
            .iter()
            .find_map(|(k, v)| (k == "text/email").then_some(v))
            .and_then(|v| match v {
                Value::String(s) => Some(String::from(s)),
                _ => None,
            });

        Ok(Entrypoint {
            metadata_raw: p.metadata,
            callback: p.callback,
            comment_size: p.comment_allowed,
            min: p.min_sendable,
            max: p.max_sendable,
            short_description,
            long_description,
            identifier,
            email,
            jpeg,
            png,
            currencies,
            payer,
        })
    }
}

impl Entrypoint {
    #[must_use]
    pub fn invoice<'a>(
        &'a self,
        amount: &'a super::Amount,
        comment: Option<&'a str>,
        convert: Option<&'a str>,
        payer: Option<super::PayerInformations>,
    ) -> Callback<'a> {
        Callback {
            url: &self.callback,
            amount,
            comment,
            convert,
            payer,
        }
    }
}

pub struct Callback<'a> {
    pub url: &'a url::Url,
    pub comment: Option<&'a str>,
    pub amount: &'a super::Amount,
    pub convert: Option<&'a str>,
    pub payer: Option<super::PayerInformations>,
}

impl std::fmt::Display for Callback<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pubkey = self
            .payer
            .as_ref()
            .and_then(|p| p.pubkey.as_ref().map(hex::encode));

        let payer = self
            .payer
            .as_ref()
            .map(|p| {
                serde_json::to_string(&super::serde::PayerInformations {
                    name: p.name.as_deref(),
                    pubkey: pubkey.as_deref(),
                    identifier: p.identifier.as_deref(),
                    email: p.email.as_deref(),
                    auth: p.auth.as_ref().map(|p| super::serde::PayerInformationAuth {
                        key: p.key.clone(),
                        k1: p.k1,
                        sig: p.sig,
                    }),
                })
            })
            .transpose()
            .map_err(|_| std::fmt::Error)?;

        let query = ser::Callback {
            comment: self.comment,
            amount: self.amount,
            convert: self.convert,
            payerdata: payer.as_deref(),
        };

        let querystr = serde_urlencoded::to_string(query).map_err(|_| std::fmt::Error)?;
        let sep = if self.url.query().is_some() { '&' } else { '?' };
        write!(f, "{}{sep}{querystr}", self.url)
    }
}

#[derive(Clone, Debug)]
pub struct CallbackResponse {
    pub pr: String,
    pub disposable: bool,
    pub success_action: Option<SuccessAction>,
}

#[derive(Clone, Debug)]
pub enum SuccessAction {
    Url(url::Url, String),
    Message(String),
}

impl std::str::FromStr for CallbackResponse {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a: de::CallbackResponse = serde_json::from_str(s).map_err(|_| "deserialize failed")?;

        let success_action = a
            .success_action
            .and_then(|sa| match sa.get("tag")? as &str {
                "message" => Some(SuccessAction::Message(String::from(sa.get("message")?))),
                "url" => {
                    let url = url::Url::parse(sa.get("url")?).ok()?;
                    let description = String::from(sa.get("description")?);
                    Some(SuccessAction::Url(url, description))
                }
                _ => None,
            });

        Ok(Self {
            pr: a.pr,
            disposable: a.disposable.unwrap_or(true),
            success_action,
        })
    }
}

mod ser {
    use serde::Serialize;

    #[derive(Serialize)]
    pub(super) struct Callback<'a> {
        pub comment: Option<&'a str>,
        #[serde(with = "super::super::serde::amount")]
        pub amount: &'a super::super::Amount,
        pub convert: Option<&'a str>,
        pub payerdata: Option<&'a str>,
    }
}

mod de {
    use super::super::serde::{Currency, Payer};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use url::Url;

    #[derive(Deserialize)]
    pub(super) struct Entrypoint<'a> {
        pub metadata: String,
        pub callback: Url,
        #[serde(rename = "minSendable")]
        pub min_sendable: u64,
        #[serde(rename = "maxSendable")]
        pub max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        pub comment_allowed: Option<u64>,
        #[serde(borrow)]
        pub currencies: Option<Vec<Currency<'a>>>,
        #[serde(rename = "payerData")]
        pub payer_data: Option<Payer<'a>>,
    }

    #[derive(Deserialize)]
    pub(super) struct CallbackResponse {
        pub pr: String,
        pub disposable: Option<bool>,
        #[serde(rename = "successAction")]
        pub success_action: Option<BTreeMap<String, String>>,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entrypoint_parse_base() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(parsed.callback.to_string(), "https://yuri/?o=callback");
        assert_eq!(parsed.short_description, "boneco do steve magal");
        assert_eq!(
            parsed.metadata_raw,
            "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]"
        );
        assert_eq!(parsed.min, 314);
        assert_eq!(parsed.max, 315);

        assert!(parsed.comment_size.is_none());
        assert!(parsed.long_description.is_none());
        assert!(parsed.jpeg.is_none());
        assert!(parsed.png.is_none());
        assert!(parsed.identifier.is_none());
        assert!(parsed.email.is_none());
        assert!(parsed.currencies.is_none());
        assert!(parsed.payer.is_none());
    }

    #[test]
    fn entrypoint_parse_comment_size() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "commentAllowed": 140,
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        assert_eq!(parsed.comment_size.unwrap(), 140);
    }

    #[test]
    fn entrypoint_parse_long_description() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/long-desc\", \"mochila a jato brutal incluida\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        assert_eq!(
            parsed.long_description.unwrap(),
            "mochila a jato brutal incluida"
        );
    }

    #[test]
    fn entrypoint_parse_images() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"image/png;base64\", \"Zm90b2JydXRhbA==\"],[\"image/jpeg;base64\", \"aW1hZ2VtYnJ1dGFs\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        assert_eq!(parsed.jpeg.unwrap(), b"imagembrutal");
        assert_eq!(parsed.png.unwrap(), b"fotobrutal");
    }

    #[test]
    fn entrypoint_parse_identifier() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/identifier\", \"steve@magal.brutal\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        assert_eq!(parsed.identifier.unwrap(), "steve@magal.brutal");
    }

    #[test]
    fn entrypoint_parse_email() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/email\", \"steve@magal.brutal\"]]",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        assert_eq!(parsed.email.unwrap(), "steve@magal.brutal");
    }

    #[test]
    fn entrypoint_parse_currencies() {
        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]",
            "maxSendable": 315,
            "minSendable": 314,
            "currencies": [
                {
                    "code": "BRL",
                    "name": "Reais",
                    "symbol": "R$",
                    "multiplier": 314.15,
                    "decimals": 2,
                    "convertible": {
                        "min": 100,
                        "max": 999
                    }
                },
                {
                    "code": "USD",
                    "name": "Dólar",
                    "symbol": "$",
                    "decimals": 6,
                    "multiplier": 14.5
                }
            ]
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        let currencies = parsed.currencies.unwrap();

        assert_eq!(currencies[0].code, "BRL");
        assert_eq!(currencies[0].name, "Reais");
        assert_eq!(currencies[0].symbol, "R$");
        assert_eq!(currencies[0].decimals, 2);
        assert!((currencies[0].multiplier - 314.15).abs() < f64::EPSILON);

        let convertible = currencies[0].convertible.as_ref().unwrap();
        assert_eq!(convertible.min, 100);
        assert_eq!(convertible.max, 999);

        assert_eq!(currencies[1].code, "USD");
        assert_eq!(currencies[1].name, "Dólar");
        assert_eq!(currencies[1].symbol, "$");
        assert_eq!(currencies[1].decimals, 6);
        assert!((currencies[1].multiplier - 14.5).abs() < f64::EPSILON);
        assert!(currencies[1].convertible.is_none());
    }

    #[test]
    fn entrypoint_parse_payer() {
        use super::super::{PayerRequirement, PayerRequirementAuth};

        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]",
            "maxSendable": 315,
            "minSendable": 314,
            "payerData": {
                "name": { "mandatory": true },
                "pubkey": { "mandatory": true },
                "identifier": { "mandatory": false },
                "email": { "mandatory": true },
                "auth": { "mandatory": true, "k1": "3132333132333231333132333132333132333132333132333331323132333132" },
                "outro": { "mandatory": false }
            }
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        let payer = parsed.payer.unwrap();

        assert!(matches!(payer.name.unwrap(), PayerRequirement { mandatory } if mandatory));
        assert!(matches!(payer.pubkey.unwrap(), PayerRequirement { mandatory } if mandatory));
        assert!(matches!(payer.identifier.unwrap(), PayerRequirement { mandatory } if !mandatory));
        assert!(matches!(payer.email.unwrap(), PayerRequirement { mandatory } if mandatory));
        assert!(
            matches!(payer.auth.unwrap(), PayerRequirementAuth { mandatory, k1 } if mandatory && &k1 == b"12312321312312312312312331212312")
        );

        assert_eq!(payer.others.len(), 1);
        assert!(
            matches!(payer.others.get("outro").unwrap(), PayerRequirement { mandatory } if !mandatory)
        );

        let input = r#"{
            "callback": "https://yuri?o=callback",
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"],[\"text/crazy\", \"👋🇧🇴💾\"]]",
            "maxSendable": 315,
            "minSendable": 314,
            "payerData": {}
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");
        let payer = parsed.payer.unwrap();

        assert!(payer.name.is_none());
        assert!(payer.pubkey.is_none());
        assert!(payer.identifier.is_none());
        assert!(payer.email.is_none());
        assert!(payer.auth.is_none());
        assert_eq!(payer.others.len(), 0);
    }

    #[test]
    fn callback_render_base() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(
            parsed
                .invoice(&super::super::Amount::Millisatoshis(314), None, None, None)
                .to_string(),
            "https://yuri/?o=callback&amount=314"
        );
    }

    #[test]
    fn callback_render_comment() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(
            parsed
                .invoice(
                    &super::super::Amount::Millisatoshis(314),
                    Some("comentario"),
                    None,
                    None
                )
                .to_string(),
            "https://yuri/?o=callback&comment=comentario&amount=314"
        );
    }

    #[test]
    fn callback_render_currency() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(
            parsed
                .invoice(
                    &super::super::Amount::Currency(String::from("BRL"), 314),
                    None,
                    None,
                    None
                )
                .to_string(),
            "https://yuri/?o=callback&amount=314.BRL"
        );
    }

    #[test]
    fn callback_render_convert() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(
            parsed
                .invoice(
                    &super::super::Amount::Millisatoshis(314),
                    None,
                    Some("BRL"),
                    None
                )
                .to_string(),
            "https://yuri/?o=callback&amount=314&convert=BRL"
        );
    }

    #[test]
    fn callback_render_payer() {
        let input = r#"{
            "metadata": "[[\"text/plain\", \"boneco do steve magal\"]]",
            "callback": "https://yuri?o=callback",
            "maxSendable": 315,
            "minSendable": 314
        }"#;

        let parsed: super::Entrypoint = input.as_bytes().try_into().expect("parse");

        assert_eq!(
            parsed
                .invoice(
                    &super::super::Amount::Millisatoshis(314),
                    None,
                    None,
                    Some(super::super::PayerInformations {
                        name: None,
                        pubkey: None,
                        identifier: None,
                        email: None,
                        auth: None,
                    })
                )
                .to_string(),
            "https://yuri/?o=callback&amount=314&payerdata=%7B%7D"
        );

        assert_eq!(
            parsed
                .invoice(
                    &super::super::Amount::Millisatoshis(314),
                    None,
                    None,
                    Some(super::super::PayerInformations {
                        name: Some(String::from("robson")),
                        pubkey: Some(b"publica".to_vec()),
                        identifier: Some(String::from("rob")),
                        email: Some(String::from("rob@son")),
                        auth: Some(super::super::PayerInformationAuth {
                            key: b"chave".to_vec(),
                            k1: *b"12332112312313123213123123211322",
                            sig:
                                *b"6564565465464564565465464565465464565464565465465465465464654343"
                        })
                    })
                )
                .to_string(),
            "https://yuri/?o=callback&amount=314&payerdata=%7B%22name%22%3A%22robson%22%2C%22pubkey%22%3A%227075626c696361%22%2C%22identifier%22%3A%22rob%22%2C%22email%22%3A%22rob%40son%22%2C%22auth%22%3A%7B%22key%22%3A%226368617665%22%2C%22k1%22%3A%223132333332313132333132333133313233323133313233313233323131333232%22%2C%22sig%22%3A%2236353634353635343635343634353634353635343635343634353635343635343634353635343634353635343635343635343635343635343634363534333433%22%7D%7D"
        );
    }

    #[test]
    fn callback_response_parse_base() {
        let input = r#"{ "pr": "pierre" }"#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(parsed.success_action.is_none());
        assert_eq!(parsed.pr, "pierre");
        assert!(parsed.disposable);
    }

    #[test]
    fn callback_response_parse_disposable() {
        let input = r#"{ "pr": "", "disposable": true }"#;
        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(parsed.disposable);

        let input = r#"{ "pr": "", "disposable": false }"#;
        let parsed = input.parse::<super::CallbackResponse>().expect("parse");
        assert!(!parsed.disposable);
    }

    #[test]
    fn callback_response_parse_success_actions() {
        let input =
            r#"{ "pr": "", "successAction": { "tag": "message", "message": "obrigado!" } }"#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");

        let Some(super::SuccessAction::Message(m)) = parsed.success_action else {
            panic!("bad success action");
        };

        assert_eq!(m, "obrigado!");

        let input = r#"
            { "pr": "", "successAction": { "tag": "url", "description": "valeu demais", "url": "http://eh.nois" } }
        "#;

        let parsed = input.parse::<super::CallbackResponse>().expect("parse");

        let Some(super::SuccessAction::Url(u, d)) = parsed.success_action else {
            panic!("bad success action");
        };

        assert_eq!(u.to_string(), "http://eh.nois/");
        assert_eq!(d, "valeu demais");
    }
}
