use std::io;

/// Parameters for an async ICMP ping (tokio-based).
#[derive(Clone, Debug)]
pub struct PingParams<'a> {
    pub destination: &'a str,
    pub count: u8,
    pub size: usize,
    pub ttl: u32,
    pub interval: u64,
}

/// Placeholder for tokio-based ICMP ping implementation.
///
/// This function is not yet implemented. It exists so that the module
/// structure mirrors the intended design (tokio for async ping).
/// The parameters are accepted (not ignored) so that a future
/// implementation can use them directly.
pub fn async_ping(params: &PingParams) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "ICMP ping via tokio is not yet implemented (destination={}, count={}, size={}, ttl={}, interval={})",
            params.destination, params.count, params.size, params.ttl, params.interval
        ),
    ))
}
