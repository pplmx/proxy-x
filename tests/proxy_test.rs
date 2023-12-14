use proxy_x;

#[test]
fn test_proxy() {
    proxy_x::enable_proxy("http://localhost:7890");
    proxy_x::disable_proxy();
}
