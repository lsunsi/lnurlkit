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
            move |()| {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::channel::server::Entrypoint {
                        uri: String::from("u@r:i"),
                        k1: String::from("caum"),
                        callback,
                    })
                }
            },
            |req: lnurlkit::channel::server::Callback| async move {
                Ok(match req {
                    lnurlkit::channel::server::Callback::Cancel { remoteid, k1 } => {
                        if &remoteid as &str == "idremoto" {
                            lnurlkit::channel::server::CallbackResponse::Ok
                        } else {
                            let reason = format!("Cancel/{k1}/{remoteid}");
                            lnurlkit::channel::server::CallbackResponse::Error { reason }
                        }
                    }
                    lnurlkit::channel::server::Callback::Accept {
                        remoteid,
                        private,
                        k1,
                    } => {
                        let reason = format!("Accept/{k1}/{remoteid}/{private}");
                        lnurlkit::channel::server::CallbackResponse::Error { reason }
                    }
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

    let queried = client.entrypoint(&lnurl).await.expect("query");
    let lnurlkit::client::Entrypoint::Channel(cr) = queried else {
        panic!("not pay request");
    };

    assert_eq!(&cr.core.uri as &str, "u@r:i");

    let response = cr.cancel("idremoto").await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::client::CallbackResponse::Ok
    ));

    let response = cr.cancel("iderrado").await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::client::CallbackResponse::Error { reason } if &reason as &str == "Cancel/caum/iderrado"
    ));

    let response = cr.accept("iderrado", true).await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::client::CallbackResponse::Error { reason } if &reason as &str == "Accept/caum/iderrado/true"
    ));

    let response = cr.accept("iderrado", false).await.expect("callback");

    assert!(matches!(
        response,
        lnurlkit::channel::client::CallbackResponse::Error { reason } if &reason as &str == "Accept/caum/iderrado/false"
    ));
}
