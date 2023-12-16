pub struct Callback<'a> {
    pub url: &'a url::Url,
    pub k1: &'a [u8; 32],
    pub sig: &'a [u8; 64],
    pub key: &'a str,
}

impl std::fmt::Display for Callback<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let query = ser::Callback {
            sig: self.sig,
            key: self.key,
        };

        let querystr = serde_urlencoded::to_string(query).map_err(|_| std::fmt::Error)?;
        let sep = if self.url.query().is_some() { '&' } else { '?' };
        write!(f, "{}{sep}{querystr}", self.url)
    }
}

mod ser {
    use serde::Serialize;

    #[derive(Serialize)]
    pub(super) struct Callback<'a> {
        #[serde(with = "hex::serde")]
        pub sig: &'a [u8; 64],
        #[serde(with = "hex::serde")]
        pub key: &'a str,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn callback_render() {
        let input =
            "https://site.com?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73";

        let parsed: super::super::Entrypoint = input.try_into().expect("try_into");
        assert_eq!(
            parsed
                .auth(
                    "chaves",
                    b"0123456789012345678901234567890123456789012345678901234567890123"
                )
                .to_string(),
            "https://site.com/\
            ?k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73\
            &sig=30313233343536373839303132333435363738393031323334353637383930313233343536373839303132333435363738393031323334353637383930313233\
            &key=636861766573"
        );
    }
}
