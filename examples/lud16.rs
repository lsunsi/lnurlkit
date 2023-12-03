#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = lnurlkit::client::Client::default();

    let queried = client.query("kenu@bipa.app").await.expect("address");

    println!("{queried:?}");

    let lnurlkit::client::Query::PayRequest(pr) = queried else {
        panic!("not pay request");
    };

    let invoice = pr.callback("comment", 123000).await.expect("callback");

    println!("{invoice:?}");
}
