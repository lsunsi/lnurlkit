#[test]
fn test() {
    let input = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
    let decoded = "https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b0ccd178df";

    let lnurlkit::Resolved::Url(url) = lnurlkit::resolve(input).expect("resolve") else {
        panic!("wrong resolved");
    };

    assert_eq!(url.as_str(), decoded);

    let lnurlkit::Resolved::Url(url) = lnurlkit::resolve(&input.to_lowercase()).expect("resolve")
    else {
        panic!("wrong resolved");
    };

    assert_eq!(url.as_str(), decoded);
}
