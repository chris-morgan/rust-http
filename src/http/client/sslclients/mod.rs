//! SSL client support.
//!
//! Which particular library is used depends upon the configuration used at
//! compile time; at present it can only be OpenSSL (`--cfg openssl`); without
//! that, you won't be able to use SSL (an attempt to make an HTTPS connection
//! will return an error).

#[cfg(feature = "ssl")]
pub use self::openssl::NetworkStream;
#[cfg(not(feature = "ssl"))]
pub use self::none::NetworkStream;

#[cfg(feature = "ssl")]
mod openssl;
#[cfg(not(feature = "ssl"))]
mod none;
