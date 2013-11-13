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
:Rust version: 0.9-pre (11b0784 2013-11-12 02:31:15 -0800)
:Node version: 0.10.20
:Go version: 1.1.2

``ab`` (new connections)
````````````````````````

::

   ab -n 10000 -c 1 http://127.0.0.1:8001/

(For higher concurrency, alter the value of ``-c``.)

=========== ==== ===== ====
Concurrency Node Go    Rust
=========== ==== ===== ====
1           4300  4100 4825
2           5300  9500 5640
3           5450 10000 6575
4           5650 10500 7100
8           5750 10750 7900
=========== ==== ===== ====

``ab -k`` (keep-alive) also works, but gets 5-10% *worse* performance, rather
than very significantly better, for some as-yet-unassessed reason.

``wrk`` (same connection)
`````````````````````````

*This benchmark is not currently automated and IS OUT OF DATE (the expected
change is that Rust should now be faring much better for higher concurrency).*

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

Single request performance is now distinctly better than Node and Go. This is a
Good Thing™. However, with higher concurrency, Rust is not faring as well as it
might, demonstrating a speedup of around 1.2× for concurrency of 2, rather than
the theoretical optimum of 2× (I don't understand how or why, but Go is getting
2.1× consistently!). Still, by the time it gets to concurrency of 8, speedup is
to about 1.65×, which is not bad; certainly it is better than Node.

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
