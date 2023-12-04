#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlc");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlc/callback")).expect("url");

    let router = lnurlkit::server::Server::default()
        .channel_request(
            move || {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::core::channel_request::ChannelRequest {
                        uri: String::from("u@r:i"),
                        k1: String::from("caum"),
                        callback,
                    })
                }
            },
            |(k1, remoteid)| async move {
                Ok(if remoteid == "idremoto" {
                    lnurlkit::core::channel_request::CallbackResponse::Ok
                } else {
                    lnurlkit::core::channel_request::CallbackResponse::Error(k1)
                })
            },
        )
        .build();

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let client = lnurlkit::client::Client::default();

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&query_url),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.query(&lnurl).await.expect("query");
    let lnurlkit::client::Query::ChannelRequest(cr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(cr.core.uri, "u@r:i");

    let response = cr
        .clone()
        .callback_cancel("idremoto")
        .await
        .expect("callback");

    assert!(matches!(
        response,
        lnurlkit::core::channel_request::CallbackResponse::Ok
    ));

    let response = cr.callback_cancel("iderrado").await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::core::channel_request::CallbackResponse::Error(r) if r == "caum"
    ));
}
