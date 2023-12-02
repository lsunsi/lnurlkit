pub const TAG: &str = "channelRequest";

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
    fn test() {
        let input = r#"
			{
			    "tag": "channelRequest",
			    "uri": "node_key@ip_address:port_number",
			    "callback": "https://bipa.app/callback?q=1",
			    "k1": "caum"
			}
        "#;

        let cr = miniserde::json::from_str::<super::ChannelRequest>(input).expect("parse");

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
