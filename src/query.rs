#[derive(serde::Deserialize)]
#[serde(tag = "tag", rename_all = "camelCase")]
pub enum Query {
    ChannelRequest(ChannelRequest),
}

impl TryFrom<&[u8]> for Query {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(s).map_err(|_| "deserialization failed")
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChannelRequest {
    pub uri: String,
    callback: url::Url,
    k1: String,
}

impl ChannelRequest {
    pub fn callback_accept(mut self, remoteid: &str, private: bool) -> url::Url {
        self.callback.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("private", if private { "1" } else { "0" }),
        ]);

        self.callback
    }

    pub fn callback_cancel(mut self, remoteid: &str) -> url::Url {
        self.callback.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("cancel", "1"),
        ]);

        self.callback
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

        let Ok(super::Query::ChannelRequest(cr)) = input.as_bytes().try_into() else {
        	panic!("Wrong query kind");
        };

        assert_eq!(cr.callback.to_string(), "https://bipa.app/callback?q=1");
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
