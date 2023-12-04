use crate::core;

#[derive(Clone, Default)]
pub struct Client(reqwest::Client);

impl Client {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn query(&self, s: &str) -> Result<Query, &'static str> {
        let url = core::resolve(s)?;

        let client = &self.0;
        let response = client.get(url).send().await.map_err(|_| "request failed")?;
        let text = response.text().await.map_err(|_| "body failed")?;

        text.parse::<core::Query>()
            .map_err(|_| "parse failed")
            .map(|query| match query {
                core::Query::ChannelRequest(core) => {
                    Query::ChannelRequest(ChannelRequest { client, core })
                }
                core::Query::WithdrawRequest(core) => {
                    Query::WithdrawRequest(WithdrawRequest { client, core })
                }
                core::Query::PayRequest(core) => Query::PayRequest(PayRequest { client, core }),
            })
    }
}

#[derive(Clone, Debug)]
pub enum Query<'a> {
    ChannelRequest(ChannelRequest<'a>),
    WithdrawRequest(WithdrawRequest<'a>),
    PayRequest(PayRequest<'a>),
}

#[derive(Clone, Debug)]
pub struct ChannelRequest<'a> {
    client: &'a reqwest::Client,
    pub core: core::channel_request::ChannelRequest,
}

#[derive(Clone, Debug)]
pub struct WithdrawRequest<'a> {
    client: &'a reqwest::Client,
    pub core: core::withdraw_request::WithdrawRequest,
}

#[derive(Clone, Debug)]
pub struct PayRequest<'a> {
    client: &'a reqwest::Client,
    pub core: core::pay_request::PayRequest,
}

impl ChannelRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback_accept(
        self,
        remoteid: &str,
        private: bool,
    ) -> Result<core::channel_request::CallbackResponse, &'static str> {
        let callback = self.core.callback_accept(remoteid, private);

        let response = self
            .client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        let text = response.text().await.map_err(|_| "body failed")?;
        text.parse().map_err(|_| "parse failed")
    }

    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback_cancel(
        self,
        remoteid: &str,
    ) -> Result<core::channel_request::CallbackResponse, &'static str> {
        let callback = self.core.callback_cancel(remoteid);

        let response = self
            .client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        let text = response.text().await.map_err(|_| "body failed")?;
        text.parse().map_err(|_| "parse failed")
    }
}

impl WithdrawRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(
        self,
        pr: &str,
    ) -> Result<core::withdraw_request::CallbackResponse, &'static str> {
        let callback = self.core.callback(pr);

        let response = self
            .client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        let text = response.text().await.map_err(|_| "body failed")?;
        text.parse().map_err(|_| "parse failed")
    }
}

impl PayRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(
        self,
        comment: &str,
        millisatoshis: u64,
    ) -> Result<core::pay_request::CallbackResponse, &'static str> {
        let callback = self.core.callback(comment, millisatoshis);

        let response = self
            .client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        let text = response.text().await.map_err(|_| "body failed")?;
        text.parse().map_err(|_| "parse failed")
    }
}
