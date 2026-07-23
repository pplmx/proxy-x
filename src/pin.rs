use std::io;

/// Re-export the shared parameter type so the `pin` command uses the same
/// fields as `ping`.
pub use crate::ping::PingParams;

/// Send ICMP ECHO_REQUEST to network hosts.
///
/// This currently delegates to the synchronous [`crate::ping::ping`]
/// implementation, which invokes the system `ping` command. A future
/// version will use tokio for true async I/O with raw sockets.
///
/// # Examples
///
/// ```no_run
/// use proxy_x::pin::{async_ping, PingParams};
///
/// let params = PingParams {
///     destination: "example.com",
///     count: 4,
///     size: 56,
///     ttl: 64,
///     interval: 1000,
/// };
/// async_ping(&params).expect("ping failed");
/// ```
pub fn async_ping(params: &PingParams) -> io::Result<()> {
    crate::ping::ping(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_ping_successful() {
        let params = PingParams {
            destination: "127.0.0.1",
            count: 1,
            size: 56,
            ttl: 64,
            interval: 1000,
        };
        let result = async_ping(&params);
        assert!(
            result.is_ok(),
            "async_ping to 127.0.0.1 should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_async_ping_fails_on_invalid_destination() {
        let params = PingParams {
            destination: "this-host-definitely-does-not-exist.invalid",
            count: 1,
            size: 56,
            ttl: 64,
            interval: 1000,
        };
        let result = async_ping(&params);
        assert!(result.is_err(), "async_ping to invalid host should fail");
    }
}
