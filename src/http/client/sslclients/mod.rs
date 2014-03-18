//! SSL client support.
//!
//! Which particular library is used depends upon the configuration used at
//! compile time; it can be OpenSSL (`--cfg openssl`), NSS (`--cfg nss`) or
//! neither. Of course, if it's neither, you won't be able to use SSL.

#[cfg(openssl)]
pub use self::openssl::NetworkStream;
#[cfg(nss)]
pub use self::nssssl::NetworkStream;
#[cfg(not(openssl), not(nss))]
pub use self::none::NetworkStream;

#[cfg(openssl)]
mod openssl;
#[cfg(nss)]
mod nssssl;
#[cfg(not(openssl), not(nss))]
mod none;
