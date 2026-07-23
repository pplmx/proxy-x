use std::io;

/// Parameters for a synchronous ICMP ping.
///
/// These map directly to the `ping` CLI options. The function currently
/// returns `Unsupported` because the pnet-backed implementation has not
/// been written yet, but the signature is correct so that callers pass
/// the full set of parameters instead of silently dropping them.
#[derive(Clone, Debug)]
pub struct PingParams<'a> {
    pub destination: &'a str,
    pub count: u8,
    pub size: usize,
    pub ttl: u32,
    pub interval: u64,
}

/// Placeholder for pnet-based ICMP ping implementation.
///
/// This function is not yet implemented. It exists so that the module
/// structure mirrors the intended design (pnet for synchronous ping).
/// The parameters are accepted (not ignored) so that a future
/// implementation can use them directly.
pub fn ping(params: &PingParams) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "ICMP ping via pnet is not yet implemented (destination={}, count={}, size={}, ttl={}, interval={})",
            params.destination, params.count, params.size, params.ttl, params.interval
        ),
    ))
}
