Comparisons
===========

Performance is important. Here are some comparisons with other languages and
environments.

Note that these examples may not be *precisely* equivalent to their Rust
counterparts as their server frameworks may insert extra headers. They should
not be using such significant differences as chunked encodings, though, which
would really render the comparison unfair.

The examples shown below are not in any way scientific tests; I have not run
them in any controlled environment whatsoever—just on my own personal machine.

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
:Rust version: 0.8 (a94158c 2013-09-26 18:46:10 -0700)
:Node version: 0.10.19
:Go version: 1.1.2

``ab`` (new connections)
````````````````````````

::

   ab -n 10000 -c 1 http://127.0.0.1:8001/

(For higher concurrency, alter the value of ``-c``.)

=========== ==== ===== ====
Concurrency Node Go    Rust
=========== ==== ===== ====
1           4150  3920 4300
2           5200  9000 4520
3           5350  9650 5200
4           5400  9750 5280
8           5670 10200 5450
=========== ==== ===== ====

``ab -k`` (keep-alive) also works, but gets 5-10% *worse* performance, rather
than very significantly better, for some as-yet-unassessed reason.

``wrk`` (same connection)
`````````````````````````

*This benchmark is not currently automated, but it is, at present, up to date.*

Ten seconds of benchmarking, with one connection kept per thread.

::

   wrk --connections 1 --duration 10s --threads 1 http://127.0.0.1:8001/

(For higher concurrency, alter the value of both ``-c`` and ``-t``.)

=========== ===== ===== =====
Concurrency Node  Go    Rust
=========== ===== ===== =====
1           10400  8600 10100
2           11400 21000 12300
3           11900 22000 13300
8           12400 21000 15000
=========== ===== ===== =====

Conclusions
===========

Single request performance is now pretty much on par with Node and Go (often
just a tad better). This is a Good Thing™. However, with higher concurrency,
Rust is fairing almost as badly as Node, not demonstrating the expected speedup
(as demonstrated very well by Go) of a factor of 2 on a dual-core machine. That
suggests there are areas in which the Rust scheduler can improve very
significantly.

Further tests need to be done, of course, benchmarking other servers for
comparison and other tech stacks, to ensure that it's all fair.

There is scope for further performance increases at all levels, among which I
include the Rust scheduler, the TCP library and writer interface (that is,
mostly, the libuv interfacing, things like requiring fewer allocations) and my
server library. There are a number of things known to be suboptimal which will
be improved. (Request reading is still not great, for example, and accounts for
about a third of the time taken with both ``wrk`` and ``ab``.)

Although my server library claims to be HTTP/1.1, it is by no means a compliant
HTTP/1.1 server; many things are not yet implemented (e.g. full request
reading, transfer codings). However, the general framework is in place and
general performance characteristics (as far as rust-http is concerned,
excluding improvements in the Rust scheduler) are not expected to change
radically. (By that I mean that I don’t believe any new things I add should
slow it down enormously.)
