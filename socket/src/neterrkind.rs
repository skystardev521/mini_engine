use std::io::ErrorKind;

pub enum NetErrorKind {
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

pub fn as_neterrkind(error: ErrorKind) -> NetErrorKind {
    return match error {
        ErrorKind::NotFound => NetErrorKind::NotFound,
        ErrorKind::PermissionDenied => NetErrorKind::PermissionDenied,
        ErrorKind::ConnectionRefused => NetErrorKind::ConnectionRefused,
        ErrorKind::ConnectionReset => NetErrorKind::ConnectionReset,
        ErrorKind::ConnectionAborted => NetErrorKind::ConnectionAborted,
        ErrorKind::NotConnected => NetErrorKind::NotConnected,
        ErrorKind::AddrInUse => NetErrorKind::AddrInUse,
        ErrorKind::AddrNotAvailable => NetErrorKind::AddrNotAvailable,
        ErrorKind::BrokenPipe => NetErrorKind::BrokenPipe,
        ErrorKind::AlreadyExists => NetErrorKind::AlreadyExists,
        ErrorKind::WouldBlock => NetErrorKind::WouldBlock,
        ErrorKind::InvalidInput => NetErrorKind::InvalidInput,
        ErrorKind::InvalidData => NetErrorKind::InvalidData,
        ErrorKind::TimedOut => NetErrorKind::TimedOut,
        ErrorKind::WriteZero => NetErrorKind::WriteZero,
        ErrorKind::Interrupted => NetErrorKind::Interrupted,
        ErrorKind::UnexpectedEof => NetErrorKind::UnexpectedEof,
        ErrorKind::Other => NetErrorKind::Other,
        _=> NetErrorKind::Other,
    };
}
