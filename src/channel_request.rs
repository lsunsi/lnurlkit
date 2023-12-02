pub const TAG: &str = "channelRequest";

#[derive(Clone, Debug)]
pub struct ChannelRequest<'a> {
    client: &'a reqwest::Client,
    callback: crate::serde::Url,
    pub uri: String,
    k1: String,
}

pub(crate) fn build<'a>(
    s: &str,
    client: &'a reqwest::Client,
) -> Result<ChannelRequest<'a>, &'static str> {
    #[derive(miniserde::Deserialize)]
    struct Deserialized {
        callback: crate::serde::Url,
        uri: String,
        k1: String,
    }

    let d: Deserialized = miniserde::json::from_str(s).map_err(|_| "deserialize failed")?;

    Ok(ChannelRequest {
        client,
        callback: d.callback,
        uri: d.uri,
        k1: d.k1,
    })
}

impl ChannelRequest<'_> {
    pub async fn callback_accept(
        mut self,
        remoteid: &str,
        private: bool,
    ) -> Result<(), &'static str> {
        self.callback.0.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("private", if private { "1" } else { "0" }),
        ]);

        self.client
            .get(self.callback.0)
            .send()
            .await
            .map_err(|_| "request failed")?;

        Ok(())
    }

    pub async fn callback_cancel(mut self, remoteid: &str) -> Result<(), &'static str> {
        self.callback.0.query_pairs_mut().extend_pairs([
            ("k1", &self.k1 as &str),
            ("remoteid", remoteid),
            ("cancel", "1"),
        ]);

        self.client
            .get(self.callback.0)
            .send()
            .await
            .map_err(|_| "request failed")?;

        Ok(())
    }
}
