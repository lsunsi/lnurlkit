#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = lnurlkit::Lnurl::default();

    let pr = client.address("kenu@bipa.app").await.expect("address");

    println!("{pr:?}");

    let invoice = pr
        .generate_invoice("comment", 123000)
        .await
        .expect("callback");

    println!("{invoice:?}");
}
