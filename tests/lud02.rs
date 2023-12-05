#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlc");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlc/callback")).expect("url");

    let router = lnurlkit::Server::default()
        .channel_request(
            move || {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::channel::Query {
                        uri: String::from("u@r:i"),
                        k1: String::from("caum"),
                        callback,
                    })
                }
            },
            |(k1, remoteid, action)| async move {
                Ok(if remoteid == "idremoto" {
                    lnurlkit::channel::CallbackResponse::Ok
                } else {
                    lnurlkit::channel::CallbackResponse::Error(format!("{k1}/{action:?}"))
                })
            },
        )
        .build();

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });

    let client = lnurlkit::Client::default();

    let lnurl = bech32::encode(
        "lnurl",
        bech32::ToBase32::to_base32(&query_url),
        bech32::Variant::Bech32,
    )
    .expect("lnurl");

    let queried = client.query(&lnurl).await.expect("query");
    let lnurlkit::client::Query::Channel(cr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(cr.core.uri, "u@r:i");

    let response = cr
        .clone()
        .callback_cancel("idremoto")
        .await
        .expect("callback");

    assert!(matches!(response, lnurlkit::channel::CallbackResponse::Ok));

    let response = cr
        .clone()
        .callback_cancel("iderrado")
        .await
        .expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::CallbackResponse::Error(r) if r == "caum/Cancel"
    ));

    let response = cr
        .clone()
        .callback_accept("iderrado", true)
        .await
        .expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::CallbackResponse::Error(r) if r == "caum/Accept { private: true }"
    ));

    let response = cr
        .callback_accept("iderrado", false)
        .await
        .expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::CallbackResponse::Error(r) if r == "caum/Accept { private: false }"
    ));
}
