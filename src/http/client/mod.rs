/*!

Modules for making HTTP requests.

At present, requests must be constructed with `RequestWriter`, which does not expose a particularly
nice-looking API.

In the future there will be a more friendly API for making requests; the Python
[Requests](http://python-requests.org/) library should be a heavy influence in this.

In the mean time, what there is is not *so* bad.

Oh yeah: don't expect to conveniently make any requests which need to send a request body yet. It's
possible, but it's not elegant convenient yet. (Most notably, no transfer-encodings are supported.)

*/

pub use self::request::RequestWriter;
pub use self::response::ResponseReader;
pub use self::sslclients::NetworkStream;

pub mod request;
pub mod response;
mod sslclients;
