#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = lnurlkit::Lnurl::default();

    let queried = client
        .query("lnurl1dp68gurn8ghj7cnfwpsjuctswqhjuam9d3kz66mwdamkutmvde6hymrs9a4k2mn4cdry4t")
        .await
        .expect("query");

    println!("{queried:?}");

    let lnurlkit::Query::PayRequest(pr) = queried else {
        panic!("not pay request");
    };

    let invoice = pr
        .generate_invoice("comment", 123000)
        .await
        .expect("callback");

    println!("{invoice:?}");
}
