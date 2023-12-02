mod channel_request;
mod pay_request;
mod serde;
mod withdrawal_request;

pub struct Lnurl(url::Url);

impl std::str::FromStr for Lnurl {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

        Ok(Lnurl(url))
    }
}

pub enum Query {
    ChannelRequest(channel_request::ChannelRequest),
    WithdrawalRequest(withdrawal_request::WithdrawalRequest),
    PayRequest(pay_request::PayRequest),
}

#[derive(miniserde::Deserialize)]
struct QueryTag {
    tag: String,
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = miniserde::json::from_str::<QueryTag>(s).map_err(|_| "deserialize tag failed")?;

        if tag.tag == channel_request::TAG {
            let cr = miniserde::json::from_str(s).map_err(|_| "deserialize data failed")?;
            Ok(Query::ChannelRequest(cr))
        } else if tag.tag == withdrawal_request::TAG {
            let wr = miniserde::json::from_str(s).map_err(|_| "deserialize data failed")?;
            Ok(Query::WithdrawalRequest(wr))
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
    fn lnurl_try_from() {
        let input = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
        let lnurl: super::Lnurl = input.parse().expect("parse");

        assert_eq!(lnurl.0.to_string(), "https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b0ccd178df");
    }
}
