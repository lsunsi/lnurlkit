#[derive(Clone, Default)]
pub struct Client(reqwest::Client);

impl Client {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn query(&self, s: &str) -> Result<Response, &'static str> {
        let url = crate::resolve(s)?;

        let client = &self.0;
        let response = client.get(url).send().await.map_err(|_| "request failed")?;
        let bytes = response.bytes().await.map_err(|_| "body failed")?;

        (&bytes as &[u8])
            .try_into()
            .map_err(|_| "parse failed")
            .map(|query: crate::Response| match query {
                crate::Response::Channel(core) => Response::Channel(Channel { client, core }),
                crate::Response::Pay(core) => Response::Pay(Pay { client, core }),
                crate::Response::Withdraw(core) => Response::Withdraw(Withdraw { client, core }),
            })
    }
}

#[derive(Clone, Debug)]
pub enum Response<'a> {
    Channel(Channel<'a>),
    Pay(Pay<'a>),
    Withdraw(Withdraw<'a>),
}

#[derive(Clone, Debug)]
pub struct Channel<'a> {
    client: &'a reqwest::Client,
    pub core: crate::channel::client::Response,
}

#[derive(Clone, Debug)]
pub struct Pay<'a> {
    client: &'a reqwest::Client,
    pub core: crate::pay::client::Response,
}

#[derive(Clone, Debug)]
pub struct Withdraw<'a> {
    client: &'a reqwest::Client,
    pub core: crate::withdraw::client::Response,
}

impl Channel<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn callback_accept(
        &self,
        remoteid: &str,
        private: bool,
    ) -> Result<crate::channel::client::CallbackResponse, &'static str> {
        let callback = self.core.callback_accept(remoteid, private);

        let response = self
            .client
            .get(callback.to_string())
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
        &self,
        remoteid: &str,
    ) -> Result<crate::channel::client::CallbackResponse, &'static str> {
        let callback = self.core.callback_cancel(remoteid);

        let response = self
            .client
            .get(callback.to_string())
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
        &self,
        millisatoshis: u64,
        comment: Option<&str>,
    ) -> Result<crate::pay::client::CallbackResponse, &'static str> {
        let callback = self.core.callback(millisatoshis, comment);

        let response = self
            .client
            .get(callback.to_string())
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
        &self,
        pr: &str,
    ) -> Result<crate::withdraw::client::CallbackResponse, &'static str> {
        let callback = self.core.callback(pr);

        let response = self
            .client
            .get(callback.to_string())
            .send()
            .await
            .map_err(|_| "request failed")?;

        let text = response.text().await.map_err(|_| "body failed")?;
        text.parse().map_err(|_| "parse failed")
    }
}
