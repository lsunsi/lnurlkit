#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub callback: url::Url,
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

impl TryFrom<Entrypoint> for Vec<u8> {
    type Error = &'static str;

    fn try_from(r: Entrypoint) -> Result<Self, Self::Error> {
        use base64::{prelude::BASE64_STANDARD, Engine};

        let metadata = serde_json::to_string(
            &[
                Some(("text/plain", r.short_description.clone())),
                r.long_description
                    .as_ref()
                    .map(|s| ("text/long-desc", s.clone())),
                r.jpeg
                    .as_ref()
                    .map(|s| ("image/jpeg;base64", BASE64_STANDARD.encode(s))),
                r.png
                    .as_ref()
                    .map(|s| ("image/png;base64", BASE64_STANDARD.encode(s))),
                r.identifier
                    .as_ref()
                    .map(|s| ("text/identifier", s.clone())),
                r.email.as_ref().map(|s| ("text/email", s.clone())),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )
        .map_err(|_| "serialize failed")?;

        serde_json::to_vec(&ser::Entrypoint {
            tag: super::TAG,
            metadata,
            callback: &r.callback,
            min_sendable: r.min,
            max_sendable: r.max,
            comment_allowed: r.comment_size.unwrap_or(0),
            currencies: r.currencies.as_ref().map(|cs| {
                cs.iter()
                    .map(|c| super::serde::Currency {
                        code: &c.code,
                        name: &c.name,
                        symbol: &c.symbol,
                        decimals: c.decimals,
                        multiplier: c.multiplier,
                        convertible: c.convertible.as_ref().map(|c| {
                            super::serde::CurrencyConvertible {
                                min: c.min,
                                max: c.max,
                            }
                        }),
                    })
                    .collect()
            }),
            payer: r.payer.as_ref().map(|p| super::serde::Payer {
                name: p.name.as_ref().map(|p| super::serde::PayerRequirement {
                    mandatory: p.mandatory,
                }),
                pubkey: p.pubkey.as_ref().map(|p| super::serde::PayerRequirement {
                    mandatory: p.mandatory,
                }),
                identifier: p
                    .identifier
                    .as_ref()
                    .map(|p| super::serde::PayerRequirement {
                        mandatory: p.mandatory,
                    }),
                email: p.email.as_ref().map(|p| super::serde::PayerRequirement {
                    mandatory: p.mandatory,
                }),
                auth: p.auth.as_ref().map(|p| super::serde::PayerRequirementAuth {
                    mandatory: p.mandatory,
                    k1: p.k1,
                }),
                others: p
                    .others
                    .iter()
                    .map(|(k, v)| {
                        (
                            k as &str,
                            super::serde::PayerRequirement {
                                mandatory: v.mandatory,
                            },
                        )
                    })
                    .collect(),
            }),
        })
        .map_err(|_| "serialize failed")
    }
}

pub struct Callback {
    pub amount: super::Amount,
    pub comment: Option<String>,
    pub convert: Option<String>,
    pub payer: Option<super::PayerInformations>,
}

impl<'a> TryFrom<&'a str> for Callback {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        serde_urlencoded::from_str::<de::Callback>(s)
            .map_err(|_| "deserialize failed")
            .and_then(|cb| {
                Ok(Callback {
                    amount: cb.amount,
                    comment: cb.comment.map(String::from),
                    convert: cb.convert.map(String::from),
                    payer: cb
                        .payerdata
                        .map(|pd| {
                            serde_json::from_str::<super::serde::PayerInformations>(&pd)
                                .map_err(|_| "deserialize payer failed")
                                .and_then(|pi| {
                                    Ok(super::PayerInformations {
                                        name: pi.name.map(String::from),
                                        pubkey: pi
                                            .pubkey
                                            .map(hex::decode)
                                            .transpose()
                                            .map_err(|_| "deserialize pubkey failed")?,
                                        identifier: pi.identifier.map(String::from),
                                        email: pi.email.map(String::from),
                                        auth: pi.auth.map(|pia| super::PayerInformationAuth {
                                            key: pia.key,
                                            k1: pia.k1,
                                            sig: pia.sig,
                                        }),
                                    })
                                })
                        })
                        .transpose()?,
                })
            })
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

impl std::fmt::Display for CallbackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let success_action = self.success_action.as_ref().map(|sa| {
            let mut map = std::collections::BTreeMap::new();

            match sa {
                SuccessAction::Message(m) => {
                    map.insert("tag", "message");
                    map.insert("message", m);
                }
                SuccessAction::Url(u, d) => {
                    map.insert("tag", "url");
                    map.insert("description", d);
                    map.insert("url", u.as_str());
                }
            }

            map
        });

        let cr = ser::CallbackResponse {
            success_action,
            disposable: self.disposable,
            pr: &self.pr,
        };

        f.write_str(&serde_json::to_string(&cr).map_err(|_| std::fmt::Error)?)
    }
}

mod ser {
    use super::super::serde::{Currency, Payer};
    use serde::Serialize;
    use std::collections::BTreeMap;
    use url::Url;

    #[derive(Serialize)]
    pub(super) struct Entrypoint<'a> {
        pub tag: &'static str,
        pub metadata: String,
        pub callback: &'a Url,
        #[serde(rename = "minSendable")]
        pub min_sendable: u64,
        #[serde(rename = "maxSendable")]
        pub max_sendable: u64,
        #[serde(rename = "commentAllowed")]
        pub comment_allowed: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub currencies: Option<Vec<Currency<'a>>>,
        #[serde(rename = "payerData", skip_serializing_if = "Option::is_none")]
        pub payer: Option<Payer<'a>>,
    }

    #[derive(Serialize)]
    pub(super) struct CallbackResponse<'a> {
        pub pr: &'a str,
        pub disposable: bool,
        #[serde(rename = "successAction")]
        pub success_action: Option<BTreeMap<&'static str, &'a str>>,
    }
}

mod de {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct Callback<'a> {
        pub comment: Option<&'a str>,
        #[serde(with = "super::super::serde::amount")]
        pub amount: super::super::Amount,
        pub convert: Option<&'a str>,
        pub payerdata: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entrypoint_render_base() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: None,
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn entrypoint_render_comment_size() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: Some(140),
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: None,
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":140}"#
        );
    }

    #[test]
    fn entrypoint_render_long_description() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: Some(String::from("mochila a jato brutal incluida")),
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: None,
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"text/long-desc\",\"mochila a jato brutal incluida\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn entrypoint_render_images() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: None,
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn entrypoint_render_identifier() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: None,
            min: 314,
            max: 315,
            identifier: Some(String::from("steve@magal.brutal")),
            email: None,
            currencies: None,
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"],[\"text/identifier\",\"steve@magal.brutal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn entrypoint_render_email() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: Some(b"imagembrutal".to_vec()),
            png: Some(b"fotobrutal".to_vec()),
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: Some(String::from("steve@magal.brutal")),
            currencies: None,
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"],[\"image/jpeg;base64\",\"aW1hZ2VtYnJ1dGFs\"],[\"image/png;base64\",\"Zm90b2JydXRhbA==\"],[\"text/email\",\"steve@magal.brutal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0}"#
        );
    }

    #[test]
    fn entrypoint_render_currencies() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: Some(vec![
                super::super::Currency {
                    code: String::from("BRL"),
                    name: String::from("Reais"),
                    symbol: String::from("R$"),
                    decimals: 2,
                    multiplier: 314.15,
                    convertible: Some(super::super::CurrencyConvertible { min: 100, max: 999 }),
                },
                super::super::Currency {
                    code: String::from("USD"),
                    name: String::from("Dolar"),
                    symbol: String::from("$"),
                    decimals: 6,
                    multiplier: 123.321,
                    convertible: None,
                },
            ]),
            payer: None,
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0,"currencies":[{"code":"BRL","name":"Reais","symbol":"R$","decimals":2,"multiplier":314.15,"convertible":{"min":100,"max":999}},{"code":"USD","name":"Dolar","symbol":"$","decimals":6,"multiplier":123.321}]}"#
        );
    }

    #[test]
    fn entrypoint_render_payer() {
        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: None,
            payer: Some(super::super::PayerRequirements {
                name: Some(super::super::PayerRequirement { mandatory: false }),
                pubkey: Some(super::super::PayerRequirement { mandatory: true }),
                identifier: Some(super::super::PayerRequirement { mandatory: false }),
                email: Some(super::super::PayerRequirement { mandatory: true }),
                auth: Some(super::super::PayerRequirementAuth {
                    mandatory: false,
                    k1: *b"12312321312312312312312331212312",
                }),
                others: [(
                    String::from("outro"),
                    super::super::PayerRequirement { mandatory: false },
                )]
                .into_iter()
                .collect(),
            }),
        };

        assert_eq!(
            Vec::<u8>::try_from(query.clone()).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0,"payerData":{"name":{"mandatory":false},"pubkey":{"mandatory":true},"identifier":{"mandatory":false},"email":{"mandatory":true},"auth":{"mandatory":false,"k1":"3132333132333231333132333132333132333132333132333331323132333132"},"outro":{"mandatory":false}}}"#
        );

        let query = super::Entrypoint {
            callback: url::Url::parse("https://yuri?o=callback").expect("url"),
            short_description: String::from("boneco do steve magal"),
            long_description: None,
            jpeg: None,
            png: None,
            comment_size: None,
            min: 314,
            max: 315,
            identifier: None,
            email: None,
            currencies: None,
            payer: Some(super::super::PayerRequirements {
                name: None,
                pubkey: None,
                identifier: None,
                email: None,
                auth: None,
                others: std::collections::HashMap::new(),
            }),
        };

        assert_eq!(
            Vec::<u8>::try_from(query).unwrap(),
            br#"{"tag":"payRequest","metadata":"[[\"text/plain\",\"boneco do steve magal\"]]","callback":"https://yuri/?o=callback","minSendable":314,"maxSendable":315,"commentAllowed":0,"payerData":{}}"#
        );
    }

    #[test]
    fn callback_parse_base() {
        let input = "amount=314";
        let parsed: super::Callback = input.try_into().expect("parse");

        assert!(matches!(
            parsed.amount,
            super::super::Amount::Millisatoshis(314)
        ));
        assert!(parsed.comment.is_none());
    }

    #[test]
    fn callback_parse_comment() {
        let input = "amount=314&comment=comentario";
        let parsed: super::Callback = input.try_into().expect("parse");

        assert!(matches!(
            parsed.amount,
            super::super::Amount::Millisatoshis(314)
        ));
        assert_eq!(parsed.comment.unwrap(), "comentario");
    }

    #[test]
    fn callback_parse_currency() {
        let input = "amount=314.BRL";
        let parsed: super::Callback = input.try_into().expect("parse");

        assert!(matches!(
            parsed.amount,
            super::super::Amount::Currency(c, 314) if c == "BRL"
        ));
        assert!(parsed.comment.is_none());
    }

    #[test]
    fn callback_parse_convert() {
        let input = "amount=314&convert=BRL";
        let parsed: super::Callback = input.try_into().expect("parse");
        assert_eq!(parsed.convert.unwrap(), "BRL");
    }

    #[test]
    fn callback_parse_payer() {
        let input = "amount=314&payerdata=%7B%22name%22%3A%22robson%22%2C%22pubkey%22%3A%227075626c696361%22%2C%22identifier%22%3A%22rob%22%2C%22email%22%3A%22rob%40son%22%2C%22auth%22%3A%7B%22key%22%3A%226368617665%22%2C%22k1%22%3A%223132333332313132333132333133313233323133313233313233323131333232%22%2C%22sig%22%3A%2236353634353635343635343634353634353635343635343634353635343635343634353635343634353635343635343635343635343635343634363534333433%22%7D%7D";
        let parsed: super::Callback = input.try_into().expect("parse");
        let payer = parsed.payer.unwrap();

        assert_eq!(payer.name.unwrap(), "robson");
        assert_eq!(payer.pubkey.unwrap(), b"publica");
        assert_eq!(payer.identifier.unwrap(), "rob");
        assert_eq!(payer.email.unwrap(), "rob@son");

        let auth = payer.auth.unwrap();
        assert_eq!(auth.key, b"chave");
        assert_eq!(auth.k1, *b"12332112312313123213123123211322");
        assert_eq!(
            auth.sig,
            *b"6564565465464564565465464565465464565464565465465465465464654343"
        );
    }

    #[test]
    fn callback_response_render_base() {
        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: None,
            disposable: true,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":true,"successAction":null}"#
        );
    }

    #[test]
    fn callback_response_render_disposable() {
        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: None,
            disposable: false,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":false,"successAction":null}"#
        );
    }

    #[test]
    fn callback_response_render_success_actions() {
        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: Some(super::SuccessAction::Message(String::from("obrigado!"))),
            disposable: false,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":false,"successAction":{"message":"obrigado!","tag":"message"}}"#
        );

        let input = super::CallbackResponse {
            pr: String::from("pierre"),
            success_action: Some(super::SuccessAction::Url(
                url::Url::parse("http://recibo").expect("url"),
                String::from("segue recibo"),
            )),
            disposable: false,
        };

        assert_eq!(
            input.to_string(),
            r#"{"pr":"pierre","disposable":false,"successAction":{"description":"segue recibo","tag":"url","url":"http://recibo/"}}"#
        );
    }
}
