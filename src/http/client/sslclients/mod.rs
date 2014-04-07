//! SSL client support.
//!
//! Which particular library is used depends upon the configuration used at
//! compile time; at present it can only be OpenSSL (`--cfg openssl`); without
//! that, you won't be able to use SSL (an attempt to make an HTTPS connection
//! will return an error).

#[cfg(openssl)]
pub use self::openssl::NetworkStream;
#[cfg(not(openssl))]
pub use self::none::NetworkStream;

#[cfg(openssl)]
mod openssl;
#[cfg(not(openssl))]
mod none;
