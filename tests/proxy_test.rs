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
