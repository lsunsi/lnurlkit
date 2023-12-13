#[derive(Clone, Default)]
pub struct Client(reqwest::Client);

impl Client {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn entrypoint(&self, s: &str) -> Result<Entrypoint, &'static str> {
        let client = &self.0;

        let url = match crate::resolve(s)? {
            crate::Resolved::Url(url) => url,
            crate::Resolved::Withdraw(_, core) => {
                return Ok(Entrypoint::Withdraw(Withdraw { client, core }))
            }
        };

        let response = client.get(url).send().await.map_err(|_| "request failed")?;
        let bytes = response.bytes().await.map_err(|_| "body failed")?;

        (&bytes as &[u8])
            .try_into()
            .map_err(|_| "parse failed")
            .map(|query: crate::Entrypoint| match query {
                crate::Entrypoint::Channel(core) => Entrypoint::Channel(Channel { client, core }),
                crate::Entrypoint::Pay(core) => Entrypoint::Pay(Pay { client, core }),
                crate::Entrypoint::Withdraw(core) => {
                    Entrypoint::Withdraw(Withdraw { client, core })
                }
            })
    }
}

#[derive(Clone, Debug)]
pub enum Entrypoint<'a> {
    Channel(Channel<'a>),
    Pay(Pay<'a>),
    Withdraw(Withdraw<'a>),
}

#[derive(Clone, Debug)]
pub struct Channel<'a> {
    client: &'a reqwest::Client,
    pub core: crate::channel::client::Entrypoint,
}

#[derive(Clone, Debug)]
pub struct Pay<'a> {
    client: &'a reqwest::Client,
    pub core: crate::pay::client::Entrypoint,
}

#[derive(Clone, Debug)]
pub struct Withdraw<'a> {
    client: &'a reqwest::Client,
    pub core: crate::withdraw::client::Entrypoint,
}

impl Channel<'_> {
    /// # Errors
    ///
    /// Returns errors on network or deserialization failures.
    pub async fn accept(
        &self,
        remoteid: &str,
        private: bool,
    ) -> Result<crate::channel::client::CallbackResponse, &'static str> {
        let callback = self.core.accept(remoteid, private);

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
    pub async fn cancel(
        &self,
        remoteid: &str,
    ) -> Result<crate::channel::client::CallbackResponse, &'static str> {
        let callback = self.core.cancel(remoteid);

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
    pub async fn invoice(
        &self,
        millisatoshis: u64,
        comment: Option<&str>,
    ) -> Result<crate::pay::client::CallbackResponse, &'static str> {
        let callback = self.core.invoice(millisatoshis, comment);

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
    pub async fn submit(
        &self,
        pr: &str,
    ) -> Result<crate::withdraw::client::CallbackResponse, &'static str> {
        let callback = self.core.submit(pr);

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
