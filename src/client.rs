#[derive(Clone, Default)]
pub struct Client(reqwest::Client);

impl Client {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn query(&self, s: &str) -> Result<Query, &'static str> {
        let url = crate::core::resolve(s)?;

        let client = &self.0;
        let response = client.get(url).send().await.map_err(|_| "request failed")?;
        let text = response.text().await.map_err(|_| "body failed")?;

        text.parse::<crate::core::Query>()
            .map_err(|_| "parse failed")
            .map(|query| match query {
                crate::core::Query::PayRequest(core) => {
                    Query::PayRequest(PayRequest { client, core })
                }
                crate::core::Query::ChannelRequest(core) => {
                    Query::ChannelRequest(ChannelRequest { client, core })
                }
                crate::core::Query::WithdrawalRequest(core) => {
                    Query::WithdrawalRequest(WithdrawalRequest { client, core })
                }
            })
    }
}

#[derive(Clone, Debug)]
pub enum Query<'a> {
    PayRequest(PayRequest<'a>),
    ChannelRequest(ChannelRequest<'a>),
    WithdrawalRequest(WithdrawalRequest<'a>),
}

#[derive(Clone, Debug)]
pub struct PayRequest<'a> {
    client: &'a reqwest::Client,
    core: crate::core::pay_request::PayRequest,
}

#[derive(Clone, Debug)]
pub struct ChannelRequest<'a> {
    client: &'a reqwest::Client,
    core: crate::core::channel_request::ChannelRequest,
}

#[derive(Clone, Debug)]
pub struct WithdrawalRequest<'a> {
    client: &'a reqwest::Client,
    core: crate::core::withdrawal_request::WithdrawalRequest,
}

impl PayRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(
        self,
        comment: &str,
        millisatoshis: u64,
    ) -> Result<crate::core::pay_request::CallbackResponse, &'static str> {
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

impl ChannelRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback_accept(self, remoteid: &str, private: bool) -> Result<(), &'static str> {
        let callback = self.core.callback_accept(remoteid, private);

        self.client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        Ok(())
    }

    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback_cancel(self, remoteid: &str) -> Result<(), &'static str> {
        let callback = self.core.callback_cancel(remoteid);

        self.client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        Ok(())
    }
}

impl WithdrawalRequest<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(self, pr: &str) -> Result<(), &'static str> {
        let callback = self.core.callback(pr);

        self.client
            .get(callback)
            .send()
            .await
            .map_err(|_| "request failed")?;

        Ok(())
    }
}
