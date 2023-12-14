pub const TAG: &str = "login";

#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub url: url::Url,
    pub k1: [u8; 32],
    pub action: Option<Action>,
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Register,
    Login,
    Link,
    Auth,
}

impl TryFrom<&str> for Entrypoint {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let url = url::Url::parse(s).map_err(|_| "url parse failed")?;
        let query = url.query().ok_or("missing query")?;
        serde_urlencoded::from_str::<de::Entrypoint>(query)
            .map_err(|_| "deserialize failed")
            .map(|c| Entrypoint {
                url,
                k1: c.k1,
                action: c.action.map(|a| match a {
                    de::Action::Register => Action::Register,
                    de::Action::Login => Action::Login,
                    de::Action::Link => Action::Link,
                    de::Action::Auth => Action::Auth,
                }),
            })
    }
}

mod de {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Entrypoint {
        #[serde(with = "hex::serde")]
        pub k1: [u8; 32],
        pub action: Option<Action>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Action {
        Register,
        Login,
        Link,
        Auth,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entrypoint_string_parse_base() {
        let input =
            "https://site.com?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73";

        let parsed: super::Entrypoint = input.try_into().expect("try_into");

        assert_eq!(&parsed.k1, b"oiprazereusouobaites!vamocamigos");
        assert!(parsed.action.is_none());
        assert_eq!(
            parsed.url.as_str(),
            "https://site.com/?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73"
        );
    }

    #[test]
    fn entrypoint_string_parse_actions() {
        let input = "https://site.com?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73&action=login";
        let parsed: super::Entrypoint = input.try_into().expect("try_into");
        assert!(matches!(parsed.action, Some(super::Action::Login)));

        let input = "https://site.com?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73&action=register";
        let parsed: super::Entrypoint = input.try_into().expect("try_into");
        assert!(matches!(parsed.action, Some(super::Action::Register)));

        let input = "https://site.com?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73&action=link";
        let parsed: super::Entrypoint = input.try_into().expect("try_into");
        assert!(matches!(parsed.action, Some(super::Action::Link)));

        let input = "https://site.com?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73&action=auth";
        let parsed: super::Entrypoint = input.try_into().expect("try_into");
        assert!(matches!(parsed.action, Some(super::Action::Auth)));
    }
}
