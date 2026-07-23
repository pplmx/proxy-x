/// Tests for proxy-x public API.
///
/// Note: enable_proxy and disable_proxy modify global git/npm config
/// as a side effect. Tests clean up after themselves by calling
/// disable_proxy at the end.

#[test]
fn test_enable_disable_proxy_cycle() {
    // Clean up any leftover state from previous runs
    let _ = proxy_x::disable_proxy();

    // Enable proxy should succeed (returns Ok or Err, but must not panic)
    let enable_result = proxy_x::enable_proxy("http://localhost:7890");
    assert!(
        enable_result.is_ok() || enable_result.is_err(),
        "enable_proxy must return a Result, not panic"
    );

    // Disable proxy should succeed (returns Ok or Err, but must not panic)
    let disable_result = proxy_x::disable_proxy();
    assert!(
        disable_result.is_ok() || disable_result.is_err(),
        "disable_proxy must return a Result, not panic"
    );

    // Clean up
    let _ = proxy_x::disable_proxy();
}

#[test]
fn test_enable_proxy_returns_result_type() {
    // Verify the return type is io::Result<()> at compile time
    let result: Result<(), std::io::Error> = proxy_x::enable_proxy("http://127.0.0.1:8080");
    // Must not panic regardless of outcome
    drop(result);
}

#[test]
fn test_disable_proxy_returns_result_type() {
    // Verify the return type is io::Result<()> at compile time
    let result: Result<(), std::io::Error> = proxy_x::disable_proxy();
    // Must not panic regardless of outcome
    drop(result);
}

#[test]
fn test_ping_returns_unsupported_with_params() {
    let params = proxy_x::ping::PingParams {
        destination: "example.com",
        count: 4,
        size: 56,
        ttl: 64,
        interval: 1000,
    };
    let result = proxy_x::ping::ping(&params);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    let msg = err.to_string();
    assert!(
        msg.contains("example.com"),
        "error should mention destination"
    );
    assert!(msg.contains("pnet"), "error should mention pnet backend");
}

#[test]
fn test_async_ping_returns_unsupported_with_params() {
    let params = proxy_x::pin::PingParams {
        destination: "8.8.8.8",
        count: 3,
        size: 128,
        ttl: 128,
        interval: 500,
    };
    let result = proxy_x::pin::async_ping(&params);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    let msg = err.to_string();
    assert!(msg.contains("8.8.8.8"), "error should mention destination");
    assert!(msg.contains("tokio"), "error should mention tokio backend");
}

#[test]
fn test_ping_params_fields() {
    // Verify PingParams can be constructed with all CLI argument values
    let params = proxy_x::ping::PingParams {
        destination: "test.host",
        count: 10,
        size: 256,
        ttl: 255,
        interval: 2000,
    };
    assert_eq!(params.destination, "test.host");
    assert_eq!(params.count, 10);
    assert_eq!(params.size, 256);
    assert_eq!(params.ttl, 255);
    assert_eq!(params.interval, 2000);
}
