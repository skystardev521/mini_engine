use std::io;

pub enum ErrorKind {
    /// An entity was not found, often a file.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// The connection was refused by the remote server.
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not
    /// local.
    AddrNotAvailable,
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    /// A parameter was incorrect.
    InvalidInput,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: #variant.InvalidInput
    InvalidData,
    /// The I/O operation's timeout expired, causing it to be canceled.
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: ../../std/io/trait.Write.html#tymethod.write
    /// [`Ok(0)`]: ../../std/io/type.Result.html
    WriteZero,
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted,
    /// Any I/O error not part of this list.
    Other,
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof,
    /// net buffer is full
    NetBufferIsFull,
    /// buffer is empty
    NetBufferIsEmpty,
    /// net pack id error
    NetPackIdError,
    /// net pack size too large
    NetPackSizeTooLarge,
}

/// io:ErrorKind ---->ErrorKind
pub fn to_error_kind(error: io::ErrorKind) -> ErrorKind {
    return match error {
        io::ErrorKind::NotFound => ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
        io::ErrorKind::ConnectionRefused => ErrorKind::ConnectionRefused,
        io::ErrorKind::ConnectionReset => ErrorKind::ConnectionReset,
        io::ErrorKind::ConnectionAborted => ErrorKind::ConnectionAborted,
        io::ErrorKind::NotConnected => ErrorKind::NotConnected,
        io::ErrorKind::AddrInUse => ErrorKind::AddrInUse,
        io::ErrorKind::AddrNotAvailable => ErrorKind::AddrNotAvailable,
        io::ErrorKind::BrokenPipe => ErrorKind::BrokenPipe,
        io::ErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
        io::ErrorKind::WouldBlock => ErrorKind::WouldBlock,
        io::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
        io::ErrorKind::InvalidData => ErrorKind::InvalidData,
        io::ErrorKind::TimedOut => ErrorKind::TimedOut,
        io::ErrorKind::WriteZero => ErrorKind::WriteZero,
        io::ErrorKind::Interrupted => ErrorKind::Interrupted,
        io::ErrorKind::UnexpectedEof => ErrorKind::UnexpectedEof,
        io::ErrorKind::Other => ErrorKind::Other,
        _ => NetErrorKind::Other,
    };
}
