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

=========== ==== ==== ====
Concurrency Node Go   Rust
=========== ==== ==== ====
1           3550 3800 2600
2           4750 7000 2750
3           4850 8000 2850
4           4800 8450 3000
8           4775 9150 2950
=========== ==== ==== ====

I have not attempted ``ab`` with ``-k`` (keep-alive) as it doesn’t seem to
work. (Haven’t had time to assess why, yet.)

``wrk`` (same connection)
`````````````````````````

**THIS BENCHMARK IS CURRENTLY OUT OF DATE.** I haven't automated it yet and had
no time to run it when I updated the above figures.

Ten seconds of benchmarking, with one connection kept per thread.

::

   wrk --connections 1 --duration 10s --threads 1 http://127.0.0.1:8001/

(For higher concurrency, alter the value of both ``-c`` and ``-t``.)

=========== ===== ===== ====
Concurrency Node  Go    Rust
=========== ===== ===== ====
1            9400  9000 6900
2           11200 20500 7200
3           11600 20500 7400
=========== ===== ===== ====

Conclusions
===========

There is still a lot of work to be done; the Rust version is able to take
advantage of very little of a second core at present, and while its performance
is par for new connections its keep-alive performance is distinctly poor. This
suggests that opening a connection performs well, but reading and writing are
comparatively slow.

There is scope for further performance increases at all levels, from the
TCP library (probably even in the scheduler) to my server library. There are a
number of things known to be suboptimal which will be improved. (Request
reading is very poor, for example, and accounts for over half the time taken
with ``wrk``, and about a third with ``ab``.)

Although my server library claims to be HTTP/1.1, it is by no means a compliant
HTTP/1.1 server; many things are not yet implemented (e.g. full request
reading, transfer codings). However, the general framework is in place and
general performance characteristics are not expected to change radically. (By
that I mean that I don’t believe any new things I add should slow it down
enormously.)
