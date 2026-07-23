use std::io;

/// Placeholder for pnet-based ICMP ping implementation.
///
/// This function is not yet implemented. It exists so that the module
/// structure mirrors the intended design (pnet for synchronous ping).
pub fn ping() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "ICMP ping via pnet is not yet implemented",
    ))
}
