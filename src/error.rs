/// Crate error
#[derive(Debug)]
pub enum Error {
    /// Read error
    Read,
    /// Write error
    Write,
    /// Invalid baud rate
    InvalidBaudRate,
    /// Invalid channel error
    InvalidChannel,
    // Timeout while waiting for response
    Timeout,
    /// Parse error
    Parse,
    /// Buffer full error
    BufferFull,
    /// Timer error
    Timer,
}
