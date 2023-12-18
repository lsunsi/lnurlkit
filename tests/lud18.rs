#[tokio::test]
async fn test() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("net");

    let addr = listener.local_addr().expect("addr");

    let query_url = format!("http://{addr}/lnurlp");
    let callback_url = url::Url::parse(&format!("http://{addr}/lnurlp/callback")).expect("url");

    let router = lnurlkit::Server::default()
        .pay_request(
            move |_| {
                let callback = callback_url.clone();
                async {
                    Ok(lnurlkit::pay::server::Entrypoint {
                        callback,
                        short_description: String::from("today i become death"),
                        long_description: None,
                        jpeg: None,
                        png: None,
                        comment_size: None,
                        min: 314,
                        max: 315,
                        identifier: None,
                        email: None,
                        currencies: None,
                        payer: Some(lnurlkit::pay::PayerRequirements {
                            name: None,
                            pubkey: None,
                            identifier: Some(lnurlkit::pay::PayerRequirement { mandatory: true }),
                            email: None,
                            auth: Some(lnurlkit::pay::PayerRequirementAuth {
                                mandatory: false,
                                k1: *b"12312312312312312312321312312312",
                            }),
                            others: std::collections::HashMap::new(),
                        }),
                    })
                }
            },
            |req: lnurlkit::pay::server::Callback| async move {
                Ok(lnurlkit::pay::server::CallbackResponse {
                    pr: format!("pierre:{:?}", req.payer),
                    disposable: false,
                    success_action: None,
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
    let lnurlkit::client::Entrypoint::Pay(pr) = queried else {
        panic!("not pay request");
    };

    let payer = pr.core.payer.as_ref().unwrap();

    assert!(payer.name.is_none());
    assert!(payer.email.is_none());
    assert!(payer.pubkey.is_none());
    assert!(matches!(
        payer.auth.as_ref().unwrap(),
        lnurlkit::pay::PayerRequirementAuth { mandatory, k1 } if !*mandatory && k1 == b"12312312312312312312321312312312"
    ));
    assert!(
        matches!(payer.identifier.as_ref().unwrap(), lnurlkit::pay::PayerRequirement { mandatory } if *mandatory)
    );

    let invoice = pr
        .invoice(
            &lnurlkit::pay::Amount::Millisatoshis(314),
            Some("comment"),
            None,
            Some(lnurlkit::pay::PayerInformations {
                name: None,
                pubkey: None,
                identifier: Some(String::from("senhor")),
                email: None,
                auth: Some(lnurlkit::pay::PayerInformationAuth {
                    key: b"linkinpark".to_vec(),
                    k1: *b"12312312312312312312321312312312",
                    sig: *b"1231231231231231231232131231231212312312312312312312321312312312",
                }),
            }),
        )
        .await
        .expect("callback");

    assert_eq!(&invoice.pr as &str, "pierre:Some(PayerInformations { name: None, pubkey: None, identifier: Some(\"senhor\"), email: None, auth: Some(PayerInformationAuth { key: [108, 105, 110, 107, 105, 110, 112, 97, 114, 107], k1: [49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 50, 49, 51, 49, 50, 51, 49, 50, 51, 49, 50], sig: [49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 50, 49, 51, 49, 50, 51, 49, 50, 51, 49, 50, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 49, 50, 51, 50, 49, 51, 49, 50, 51, 49, 50, 51, 49, 50] }) })");
}
