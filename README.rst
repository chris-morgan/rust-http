Rust HTTP library
=================

.. image:: https://travis-ci.org/chris-morgan/rust-http.png?branch=master
   :target: https://travis-ci.org/chris-morgan/rust-http

This project has two parts:

1. An HTTP server

2. An HTTP client

Both are in progress; both have basic, low-level implementations in place.
Neither is complete nor yet compliant in any way.

Rust versions
-------------

I urge you to track Rust master as rust-http does, but if you really are set on
using Rust 0.9, you can use the [`rust-0.9-compatible`
branch](https://github.com/chris-morgan/rust-http/commits/rust-0.9-compatible).
It is not maintained, however; it's just the last commit that *will* work on
Rust 0.9.

Goals
-----

The goal of the present phase of this project is, quite simply, to create a
generic HTTP server and client library for Rust.

When I say “generic”, generic is what I mean: it is quite feasible to write
non-origin servers (e.g. a proxy or a gateway) with rust-http; there will
merely be a slightly higher level abstraction available for origin servers to
use.

At present this is all one crate, but it may be separated into multiple crates
(e.g. common HTTP, client, server); I am not sure about this yet.

This server is not (in the normal sense) opinionated; it provides the tools,
handles communication, the HTTP/1.1 protocol and the basic headers and then
leaves the rest to you. Things like URL routing do not belong in here; that is
the domain of a framework.

There is, however, one thing on which it has strong opinions: using the type
system. rust-http forces you to make type-safe code. This has benefits in
safety and correctness, and also typically in speed.

Getting started
---------------

This is developed on Ubuntu and is aimed at the moving target of Rust's master
branch.

Build everything::

   make all

Run one of the servers::

   build/examples/server/apache_fake

To run the client example, start one of the servers and run::

   build/examples/client http://127.0.0.1:8001/

At present, all of the example servers serve to http://127.0.0.1:8001/.

Don't expect everything to work well. The server claims HTTP/1.1, but is not
in any way compliant yet.

SSL support
-----------

rust-http can be compiled with or without SSL support.

To compile with SSL support, drop rust-openssl_ in a sibling directory of
rust-http (i.e. ``../rust-openssl`` from this file) and run its ``configure``
and ``make``. rust-http's ``configure`` will then automatically detect it and
you will get SSL support enabled.

To compile rust-http without SSL support, just don’t put rust-openssl_ where it
can find it. You'll then get an ``IoError { kind: InvalidInput, .. }`` if you
try to make an SSL request (e.g. HTTPS).

.. _rust-openssl: https://github.com/sfackler/rust-openssl

Roadmap
-------

Here are some of the things to be done. They are only *very* vaguely ordered
and treat client and server quite indiscriminately.

- Implement HTTP/1.1.

  - Handle transfer codings, especially chunked (rather essential to a server
    supporting keep-alive).

  - Read request body (start with reading it entirely, for simplicity: at
    present, one can't split up the TcpStream into a Reader and a Writer that
    can both be accessed at once; just recognise this is an efficiency and
    security flaw, to be fixed later in "read request body incrementally").
    This is necessary to do things like POST.

  - Treat headers as data, not strings.

  - Make it easy to work with things like cookies, etags and caches.

- Tests, lots of them. There's very little just now.

- Benchmarks, lots of them. This will be helpful at the level of my code and
  also at the runtime level.

- Improve the convenience and correctness of the HTTP client.

- DoS prevention: largely to do with things like connection timeouts.
  Make a test suite using pathod_.

- Efficiency/DoS prevention: read request body incrementally.

- TLS/SSL comes a long way down the path; before I would be willing to trust it
  as secure server I'd want an independent audit; for now, it's safer and
  easier to use it through a reverse proxy like Nginx and let it take care of
  SSL.

  Servo's existence will be helping this project quite a bit; it will,
  somewhere along the way, need a TLS/SSL implementation, and we'll be able to
  use that, mostly.

When most of these things are done, *then* I'll start developing my web
framework. And it'll end up blindingly fast, astonishingly safe and remarkably
convenient. But don't plan on using it in 2013.

Design (server)
---------------

(In this section, the first person pronoun refers to Chris Morgan.)

I have hitherto been by-and-large a Python developer. I started designing this
by examining Python's WSGI (`PEP 3333`_), which is considered good enough that
Ruby's Rack is directly based on it. However, I quickly came to the conclusion
that its design is optimised for a quite different use case; my goal in Rust
is to provide the entire server, rather than to interface with another server.
Certain other design decisions are incompatible with my Grand Vision. Take, for
example, this case:

   *Why use CGI variables instead of good old HTTP headers? And why mix them in
   with WSGI-defined variables?*

   Many existing web frameworks are built heavily upon the CGI spec, and
   existing web servers know how to generate CGI variables. In contrast,
   alternative ways of representing inbound HTTP information are fragmented and
   lack market share. Thus, using the CGI "standard" seems like a good way to
   leverage existing implementations. As for mixing them with WSGI variables,
   separating them would just require two dictionary arguments to be passed
   around, while providing no real benefits.

In Rust, there is no base of code already following such a convention and so we
are not tethered by this requirement. My own feeling on such matters is that
for the static typing world having such things is not beneficial, anyway. Most
web systems would have something along these lines, working with what is
effectively a ``Map<~str, ~str>``::

   response.headers["Date"] = format_http_time(now_utc())

The header ``Date`` is *known*, and is a date and time. Why not rather have it
thus?

::

   response.headers.date = now_utc()

To be certain, there may be need for unknown headers; yet even there one
probably does not wish a ``~str`` value, but a more suitable type implementing
a trait to convert to and from an appropriate string.

Note that with these examples the precise form is not determined.

The end result of these matters is that I determined not to model WSGI at all.
In the end, Go's ``net/http`` package has been my primary source of
*inspiration*, but I am creating something which is quite definitely distinct:
``net/http`` is for inspiration only, then I do my own thing. You see, Go lacks
all sorts of nice things Rust has, such as its enums and iteration on aught
beyond built-in types.

License
-------

This library is distributed under similar terms to Rust: dual licensed under
the MIT license and the Apache license (version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.

.. _PEP 3333: http://www.python.org/dev/peps/pep-3333/
.. _pathod: http://pathod.net/

.. vim:ft=rst
