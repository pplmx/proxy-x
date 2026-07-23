use std::io;

/// Placeholder for tokio-based ICMP ping implementation.
///
/// This function is not yet implemented. It exists so that the module
/// structure mirrors the intended design (tokio for async ping).
pub fn async_ping() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "ICMP ping via tokio is not yet implemented",
    ))
}
