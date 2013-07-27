Comparisons
===========

Performance is important. Here are some comparisons with other languages and
environments.

Note that these examples may not be *precisely* equivalent to their Rust
counterparts as their server frameworks may insert extra headers. They should
not be using such significant differences as chunked encodings, though, which
would really render the comparison unfair.

How to run the examples
-----------------------

Any Node examples::

   node ___.js

Any Go examples::

   go run ___.go

Results
=======

Apache fake
-----------

This test is designed to produce much the same request as the default page
Apache HTTP server serves. (Or did, before I updated from Ubuntu 13.04 to 13.10
and Apache config got broken.)

:Hardware: Over-six-year-old Core 2 Duo laptop with 4GB ("plenty") of RAM
:OS: Ubuntu 13.10 (alpha) 64-bit
:Rust version: 0.8-pre (906264b 2013-07-25 03:07:44 -0700)
:Node version: 0.10.15
:Go version: 1.1.1

Benchmarking is done with ApacheBench at present as the easiest way (yes, I
know ``ab`` is reasonably disparaged)::

   ab -n 10000 -c 1 http://127.0.0.1:8001/

Getting results from Rust with concurrency > 1 is difficult at present as it'll
normally segfault within a few thousand requests. Just keep trying...

=========== ==== ==== ====
Concurrency Node Go   Rust
=========== ==== ==== ====
1           3200 3000 3500
2           4400 6500 4000
3           4700 7400 4500
=========== ==== ==== ====
