pub mod channel_request;
pub mod pay_request;
pub mod withdraw_request;

/// # Errors
///
/// Returns error in case `s` cannot be understood.
pub fn resolve(s: &str) -> Result<url::Url, &'static str> {
    if s.starts_with("lnurl1") || s.starts_with("LNURL1") {
        resolve_bech32(s)
    } else if s.starts_with("lnurl") || s.starts_with("keyauth") {
        resolve_scheme(s)
    } else if s.contains('@') {
        resolve_address(s)
    } else {
        Err("unknown")
    }
}

fn resolve_bech32(s: &str) -> Result<url::Url, &'static str> {
    let Ok((hrp, data, _)) = bech32::decode(s) else {
        return Err("bech32 decode failed");
    };

    if hrp != "lnurl" {
        return Err("bech32 hrp invalid");
    }

    let Ok(bytes) = <Vec<u8> as bech32::FromBase32>::from_base32(&data) else {
        return Err("bech32 data is not bytes");
    };

    let Ok(text) = String::from_utf8(bytes) else {
        return Err("bech32 bytes is not string");
    };

    let Ok(url) = url::Url::parse(&text) else {
        return Err("bech32 text is not a url");
    };

    Ok(url)
}

fn resolve_scheme(s: &str) -> Result<url::Url, &'static str> {
    let s = s
        .trim_start_matches("keyauth://")
        .trim_start_matches("lnurlc://")
        .trim_start_matches("lnurlw://")
        .trim_start_matches("lnurlp://");

    let Ok(url) = url::Url::parse(&format!("https://{s}")) else {
        return Err("bad url");
    };

    Ok(url)
}

fn resolve_address(s: &str) -> Result<url::Url, &'static str> {
    let Some((identifier, domain)) = s.split_once('@') else {
        return Err("split failed");
    };

    let Ok(url) = url::Url::parse(&format!("https://{domain}/.well-known/lnurlp/{identifier}"))
    else {
        return Err("bad url");
    };

    Ok(url)
}

#[derive(Debug)]
pub enum Query {
    PayRequest(pay_request::PayRequest),
    ChannelRequest(channel_request::ChannelRequest),
    WithdrawRequest(withdraw_request::WithdrawRequest),
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[derive(miniserde::Deserialize)]
        struct Tag {
            tag: String,
        }

        let tag = miniserde::json::from_str::<Tag>(s).map_err(|_| "deserialize tag failed")?;

        if tag.tag == channel_request::TAG {
            let cr = s.parse().map_err(|_| "deserialize data failed")?;
            Ok(Query::ChannelRequest(cr))
        } else if tag.tag == withdraw_request::TAG {
            let wr = s.parse().map_err(|_| "deserialize data failed")?;
            Ok(Query::WithdrawRequest(wr))
        } else if tag.tag == pay_request::TAG {
            let pr = s.parse().map_err(|_| "deserialize data failed")?;
            Ok(Query::PayRequest(pr))
        } else {
            Err("unknown tag")
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn resolve_bech32() {
        let input = "lnurl1dp68gurn8ghj7argv4ex2tnfwvhkumelwv7hqmm0dc6p3ztw";
        assert_eq!(
            super::resolve(input).unwrap().to_string(),
            "https://there.is/no?s=poon"
        );

        let input = "LNURL1DP68GURN8GHJ7ARGV4EX2TNFWVHKUMELWV7HQMM0DC6P3ZTW";
        assert_eq!(
            super::resolve(input).unwrap().to_string(),
            "https://there.is/no?s=poon"
        );
    }

    #[test]
    fn resolve_address() {
        assert_eq!(
            super::resolve("no-spoon@there.is").unwrap().to_string(),
            "https://there.is/.well-known/lnurlp/no-spoon"
        );
    }

    #[test]
    fn resolve_schemes() {
        let input = "lnurlc://there.is/no?s=poon";
        assert_eq!(
            super::resolve(input).unwrap().to_string(),
            "https://there.is/no?s=poon"
        );

        let input = "lnurlw://there.is/no?s=poon";
        assert_eq!(
            super::resolve(input).unwrap().to_string(),
            "https://there.is/no?s=poon"
        );

        let input = "lnurlp://there.is/no?s=poon";
        assert_eq!(
            super::resolve(input).unwrap().to_string(),
            "https://there.is/no?s=poon"
        );

        let input = "keyauth://there.is/no?s=poon";
        assert_eq!(
            super::resolve(input).unwrap().to_string(),
            "https://there.is/no?s=poon"
        );
    }
}
