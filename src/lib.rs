#![allow(clippy::multiple_crate_versions)]
// socket2 from hyper and tokio

mod channel_request;
mod pay_request;
mod serde;
mod withdrawal_request;

pub use pay_request::PayRequest;

#[derive(Clone, Default)]
pub struct Lnurl(reqwest::Client);

impl Lnurl {
    /// # Errors
    ///
    /// Will return error in case `s` is not a valid lnurl,
    /// when request or parsing fails, basically anything that goes bad.
    pub async fn query(&self, s: &str) -> Result<Query, &'static str> {
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

        let response = self.0.get(url).send().await.map_err(|_| "request failed")?;
        let body = response.text().await.map_err(|_| "body failed")?;
        let query = build(&body, &self.0).map_err(|_| "parse failed")?;

        Ok(query)
    }
}

#[derive(Debug)]
pub enum Query<'a> {
    ChannelRequest(channel_request::ChannelRequest<'a>),
    WithdrawalRequest(withdrawal_request::WithdrawalRequest<'a>),
    PayRequest(pay_request::PayRequest<'a>),
}

fn build<'a>(s: &str, client: &'a reqwest::Client) -> Result<Query<'a>, &'static str> {
    #[derive(miniserde::Deserialize)]
    struct Tag {
        tag: String,
    }

    let tag = miniserde::json::from_str::<Tag>(s).map_err(|_| "deserialize tag failed")?;

    if tag.tag == channel_request::TAG {
        let cr = channel_request::build(s, client).map_err(|_| "deserialize data failed")?;
        Ok(Query::ChannelRequest(cr))
    } else if tag.tag == withdrawal_request::TAG {
        let wr = withdrawal_request::build(s, client).map_err(|_| "deserialize data failed")?;
        Ok(Query::WithdrawalRequest(wr))
    } else if tag.tag == pay_request::TAG {
        let pr = pay_request::build(s, client).map_err(|_| "deserialize data failed")?;
        Ok(Query::PayRequest(pr))
    } else {
        Err("unknown tag")
    }
}
