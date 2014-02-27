HTTP client plan
================

A large source of inspiration for this should be requests_, but we will head in
a quite different direction from it.

In the first stage of The Plan, I will have ``http::client`` (new code) and
``http::server`` (currently most of ``rusthttpserver``). Some things currently
in ``rusthttpserver`` which are definitely common to client and server with no
difference, such as the ``headers`` and ``method`` modules, will go in a shared
location (``http::*``, probably). *Package separation state:* complete.

Types:

- ``RequestWriter``, which satisfies ``std::io::Writer``, but also provides
  various convenience methods for managing other ways of doing things.

To begin with, the client user will formulate an ``http::client::Request``.
(Initially it will be different from ``http::server::Request``, though the two
will share quite a lot of functionality; whether it will be a different struct
in the end, I do not know; Go has its ``net/http.Request`` shared between client
and server, but then has a couple of fields either way which it is not valid to
set in certain ways for a request or a response.)

When the user has constructed his ``Request``, he will call the synchronous
``Request.send()``. (Connection pooling and persistent session management for
things like cookies and authentication is out of scope for the first version;
it can come later.) This may raise a condition (from upstream or a new
one—compare with requests' ``requests.exceptions`` which contains some
necessary things). It returns an ``Option<http::client::Response>``.

The ``Response`` has an API very strongly reminiscent of
``http::server::ResponseWriter``, but does need to be separate, being a reader
(and yes, a ``Reader`` for reading the body) rather than a writer.

Character set determination needs to be able to be done somewhere; it can
probably be a wrapper about a ``Reader``.

The initial API will be very simple, with ``Request::new(Method, Url)`` and the
use of string typing for headers::

   extern crate http;
   use http::client::Request;
   use http::method::Get;
   use extra::url::Url;

   let mut request = Request::new(Get, from_str("http://rust-lang.org"));
   request.headers.insert(~"Connection", ~"close");
   request.headers.insert(~"Referer", ~"https://google.com/");
   let mut response = request.send();
   assert_eq!(response.status, status::Ok);
   assert_eq!(response.version, (1, 1));

My initial feeling was that ``request.send()`` should consume ``request``,
putting it in ``response.request`` as immutable, but I'm not sure if that'll
work, as ``request.send()`` might return ``None``. Perhaps it would need to
return a ``Result<Response, Request>`` instead of an ``Option<Response>``.
Still got to think about how to handle that, as we can't let the Request object
get lost.

This initial version will not provide any assistance with redirect handling; to
begin with, Servo can manage that itself as we determine the best manner in
which to handle it. (I'm familiar with the specs on matters of redirection,
automatic or requiring user intervention, but I'm not certain how something
like Servo is best designed to manage it.)

.. _requests: http://python-requests.org/
