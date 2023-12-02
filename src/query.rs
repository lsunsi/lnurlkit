pub enum Query {
    ChannelRequest(ChannelRequest),
}

#[derive(miniserde::Deserialize)]
struct QueryTag {
    tag: String,
}

impl std::str::FromStr for Query {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = miniserde::json::from_str::<QueryTag>(s).map_err(|_| "deserialize tag failed")?;

        match &tag.tag as &str {
            "channelRequest" => {
                let a = miniserde::json::from_str(s).map_err(|_| "deserialize data failed")?;
                Ok(Query::ChannelRequest(a))
            }
            _ => Err("unknown tag"),
        }
    }
}

#[derive(Debug, Clone, miniserde::Deserialize)]
pub struct ChannelRequest {
    callback: crate::serde::Url,
    pub uri: String,
    k1: String,
}

impl ChannelRequest {
    pub fn callback_accept(mut self, remoteid: &str, private: bool) -> url::Url {
        self.callback.0.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("private", if private { "1" } else { "0" }),
        ]);

        self.callback.0
    }

    pub fn callback_cancel(mut self, remoteid: &str) -> url::Url {
        self.callback.0.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("cancel", "1"),
        ]);

        self.callback.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn channel_request() {
        let input = r#"
			{
			    "tag": "channelRequest",
			    "uri": "node_key@ip_address:port_number",
			    "callback": "https://bipa.app/callback?q=1",
			    "k1": "caum"
			}
        "#;

        let Ok(super::Query::ChannelRequest(cr)) = input.parse() else {
        	panic!("Wrong query kind");
        };

        assert_eq!(cr.callback.0.to_string(), "https://bipa.app/callback?q=1");
        assert_eq!(cr.uri, "node_key@ip_address:port_number");
        assert_eq!(cr.k1, "caum");

        assert_eq!(
            cr.clone().callback_accept("idremoto", false).to_string(),
            "https://bipa.app/callback?q=1&k1=caum&remoteid=idremoto&private=0"
        );

        assert_eq!(
            cr.clone().callback_accept("idremoto", true).to_string(),
            "https://bipa.app/callback?q=1&k1=caum&remoteid=idremoto&private=1"
        );

        assert_eq!(
            cr.callback_cancel("idremoto").to_string(),
            "https://bipa.app/callback?q=1&k1=caum&remoteid=idremoto&cancel=1"
        );
    }
}
