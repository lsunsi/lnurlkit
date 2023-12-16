pub struct Callback {
    pub sig: [u8; 64],
    pub key: Vec<u8>,
}

impl<'a> TryFrom<&'a str> for Callback {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        serde_urlencoded::from_str::<de::Callback>(s)
            .map_err(|_| "deserialize failed")
            .map(|cb| Callback {
                sig: cb.sig,
                key: cb.key,
            })
    }
}

mod de {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub(super) struct Callback {
        #[serde(with = "hex::serde")]
        pub sig: [u8; 64],
        #[serde(with = "hex::serde")]
        pub key: Vec<u8>,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn callback_parse() {
        let input =
            "k1=6f697072617a65726575736f756f6261697465732176616d6f63616d69676f73\
            &sig=30313233343536373839303132333435363738393031323334353637383930313233343536373839303132333435363738393031323334353637383930313233\
            &key=636861766573";

        let parsed: super::Callback = input.try_into().expect("try_into");
        assert_eq!(parsed.key, b"chaves");
        assert_eq!(
            &parsed.sig,
            b"0123456789012345678901234567890123456789012345678901234567890123"
        );
    }
}
