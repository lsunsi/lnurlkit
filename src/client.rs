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
                crate::core::Query::Channel(core) => Query::Channel(Channel { client, core }),
                crate::core::Query::Pay(core) => Query::Pay(Pay { client, core }),
                crate::core::Query::Withdraw(core) => Query::Withdraw(Withdraw { client, core }),
            })
    }
}

#[derive(Clone, Debug)]
pub enum Query<'a> {
    Channel(Channel<'a>),
    Pay(Pay<'a>),
    Withdraw(Withdraw<'a>),
}

#[derive(Clone, Debug)]
pub struct Channel<'a> {
    client: &'a reqwest::Client,
    pub core: crate::core::channel::Query,
}

#[derive(Clone, Debug)]
pub struct Pay<'a> {
    client: &'a reqwest::Client,
    pub core: crate::core::pay::Query,
}

#[derive(Clone, Debug)]
pub struct Withdraw<'a> {
    client: &'a reqwest::Client,
    pub core: crate::core::withdraw::Query,
}

impl Channel<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback_accept(
        self,
        remoteid: &str,
        private: bool,
    ) -> Result<crate::core::channel::CallbackResponse, &'static str> {
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
    ) -> Result<crate::core::channel::CallbackResponse, &'static str> {
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

impl Pay<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(
        self,
        comment: &str,
        millisatoshis: u64,
    ) -> Result<crate::core::pay::CallbackResponse, &'static str> {
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

impl Withdraw<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback(
        self,
        pr: &str,
    ) -> Result<crate::core::withdraw::CallbackResponse, &'static str> {
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
