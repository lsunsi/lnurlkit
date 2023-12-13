pub mod channel;
pub mod pay;
pub mod withdraw;

pub enum Resolved {
    Url(url::Url),
    Withdraw(url::Url, withdraw::client::Entrypoint),
}

/// # Errors
///
/// Returns error in case `s` cannot be understood.
pub fn resolve(s: &str) -> Result<Resolved, &'static str> {
    let url = if s.starts_with("lnurl1") || s.starts_with("LNURL1") {
        resolve_bech32(s)
    } else if s.starts_with("lnurl") || s.starts_with("keyauth") {
        resolve_scheme(s)
    } else if s.contains('@') {
        resolve_address(s)
    } else {
        Err("unknown")
    }?;

    let tag = url
        .query_pairs()
        .find_map(|(k, v)| (k == "tag").then_some(v));

    Ok(match tag.as_deref() {
        Some(withdraw::TAG) => match url.as_str().parse::<withdraw::client::Entrypoint>() {
            Ok(w) => Resolved::Withdraw(url, w),
            Err(_) => Resolved::Url(url),
        },
        _ => Resolved::Url(url),
    })
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
pub enum Entrypoint {
    Channel(channel::client::Entrypoint),
    Pay(pay::client::Entrypoint),
    Withdraw(withdraw::client::Entrypoint),
}

impl TryFrom<&[u8]> for Entrypoint {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        #[derive(serde::Deserialize)]
        struct Tag {
            tag: String,
        }

        let tag = serde_json::from_slice::<Tag>(s).map_err(|_| "deserialize tag failed")?;

        if tag.tag == channel::TAG {
            let cr = s.try_into().map_err(|_| "deserialize data failed")?;
            Ok(Entrypoint::Channel(cr))
        } else if tag.tag == pay::TAG {
            let pr = s.try_into().map_err(|_| "deserialize data failed")?;
            Ok(Entrypoint::Pay(pr))
        } else if tag.tag == withdraw::TAG {
            let wr = s.try_into().map_err(|_| "deserialize data failed")?;
            Ok(Entrypoint::Withdraw(wr))
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
        let super::Resolved::Url(url) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/no?s=poon");

        let input = "LNURL1DP68GURN8GHJ7ARGV4EX2TNFWVHKUMELWV7HQMM0DC6P3ZTW";
        let super::Resolved::Url(url) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/no?s=poon");
    }

    #[test]
    fn resolve_address() {
        let super::Resolved::Url(url) = super::resolve("no-spoon@there.is").unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/.well-known/lnurlp/no-spoon");
    }

    #[test]
    fn resolve_schemes() {
        let input = "lnurlc://there.is/no?s=poon";
        let super::Resolved::Url(url) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/no?s=poon");

        let input = "lnurlw://there.is/no?s=poon";
        let super::Resolved::Url(url) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/no?s=poon");

        let input = "lnurlp://there.is/no?s=poon";
        let super::Resolved::Url(url) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/no?s=poon");

        let input = "keyauth://there.is/no?s=poon";
        let super::Resolved::Url(url) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(url.as_str(), "https://there.is/no?s=poon");
    }

    #[test]
    fn resolve_fast_withdraw() {
        let input = "lnurlw://there.is/no\
            ?s=poon\
            &tag=withdrawRequest\
            &k1=caum\
            &minWithdrawable=314\
            &maxWithdrawable=315\
            &defaultDescription=descrical\
            &callback=https://call.back";

        let super::Resolved::Withdraw(url, _) = super::resolve(input).unwrap() else {
            panic!("expected resolved url");
        };

        assert_eq!(
            url.as_str(),
            "https://there.is/no\
            ?s=poon\
            &tag=withdrawRequest\
            &k1=caum\
            &minWithdrawable=314\
            &maxWithdrawable=315\
            &defaultDescription=descrical\
            &callback=https://call.back"
        );
    }
}
