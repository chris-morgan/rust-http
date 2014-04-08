'''Comparison benchmarker.'''

import os
import subprocess
import sys
import time
import urllib2
from contextlib import contextmanager


@contextmanager
def tempmsg(msg):
    '''
    Write a message to the current line of the terminal and then erase the
    entire line when exiting the block.
    '''
    try:
        sys.stdout.write(msg)
        sys.stdout.flush()
        yield
    finally:
        sys.stdout.write('[2K\r')
        sys.stdout.flush()


class ServerRunner(object):
    '''Abstract base class for server benchmark runners.'''

    PLATFORM = NotImplemented

    # When starting, check that the server is serving on HTTP every N seconds
    START_CHECK_FREQUENCY = 0.2  # seconds

    START_CHECK_TIMEOUT = 2  # seconds

    # How long should we wait after killing before going on to the next thing?
    KILL_DELAY = 0.5  # seconds

    def __init__(self, name, source, build_dir, hostname, port):
        self.name = name
        self.source = source
        self.build_dir = build_dir
        self.hostname = hostname
        self.port = port

    @property
    def root_url(self):
        '''Get the root URL that the server is serving to.'''
        if self.port == 80:
            return 'http://{}/'.format(self.hostname)
        else:
            return 'http://{}:{}/'.format(self.hostname, self.port)

    def get_server_process_details(self):
        '''Get the (image_name, args) of the subprocess to spawn.'''
        raise NotImplementedError()

    def compile_server(self):
        '''Compile the server, if such a step is necessary.'''

        # This method left intentionally blank.

    def spawn_server(self):
        '''
        Start running the server.

        :returns: the :class:`subprocess.Popen` object pertaining to it.
        '''
        args = self.get_server_process_details()
        process = subprocess.Popen(args,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        for _ in xrange(int(self.START_CHECK_TIMEOUT /
                            self.START_CHECK_FREQUENCY)):
            time.sleep(self.START_CHECK_FREQUENCY)
            try:
                urllib2.urlopen(self.root_url)
            except urllib2.URLError:
                pass
            else:
                break
        return process

    @contextmanager
    def activate(self):
        '''
        A context manager during which the server is running.

        Compilation with :meth:`compile_server` must already have been done.
        '''
        process = self.spawn_server()
        try:
            yield
        finally:
            process.kill()
            time.sleep(self.KILL_DELAY)


class GoServerRunner(ServerRunner):
    '''A runner for Go servers.'''

    PLATFORM = 'go'

    def __init__(self, *args, **kwargs):
        super(GoServerRunner, self).__init__(*args, **kwargs)
        assert self.source.endswith('.go'), 'source must end in ".go"'
        self.bin_name = 'go-' + os.path.basename(self.source[:-3])

    def compile_server(self):
        subprocess.Popen(('go', 'build',
            '-o', os.path.join(self.build_dir, self.bin_name),
            self.source)).communicate()

    def get_server_process_details(self):
        return os.path.join(self.build_dir, self.bin_name),


class NodeServerRunner(ServerRunner):
    '''A runner for Node servers.'''

    PLATFORM = 'node'

    def get_server_process_details(self):
        return 'node', self.source


class RustServerRunner(ServerRunner):
    '''A runner for Rust servers.'''

    PLATFORM = 'rust'

    # e.g. x86_64-unknown-linux-gnu
    HOST = subprocess.Popen(('rustc', '--version'),
                            stdout=subprocess.PIPE).communicate()[0].split(
                                    'host: ')[1].rstrip()

    def __init__(self, *args, **kwargs):
        super(RustServerRunner, self).__init__(*args, **kwargs)
        assert self.source.endswith('.rs'), 'source must end in ".rs"'
        # Designed for the .../x/main.rs pattern (from rustpkg), to get x.
        self.bin_name = os.path.basename(os.path.dirname(self.source))

    def compile_server(self):
        subprocess.Popen(('rustc',
            '--opt-level=3', self.source,
            #'--out-dir', self.build_dir,
            '-L', '../build', # '../build/{}/http/'.format(RustServerRunner.HOST),
            # Just in case it was built with openssl support. This should
            # really be done better, based on the Makefile contents.
            '-L', '../../rust-openssl/build',
            # Sorry, this main.rs business needs me to do this, or use rustpkg:
            '-o', os.path.join(self.build_dir, self.bin_name))).communicate()

    def get_server_process_details(self):
        return os.path.join(self.build_dir, self.bin_name),


class ServerRunnerCollection(object):
    r'''
    A collection of :class:`ServerRunner`\ s, all with the same configuration.

    This is making the assumption that all of the examples are applicable to
    all of the server classes.
    '''

    # Eek! Plenty of code duplication! Metaclasses could fix this if I felt
    # like it, but it makes it much more convoluted. I'll just wait until I do
    # it in Rust, then I can use macros for it, and *much* more conveniently.
    def __init__(self, name, skip=(), *args, **kwargs):
        if 'go' not in skip:
            self.go = GoServerRunner(
                source=name + '.go',
                name=name, *args, **kwargs)
        if 'node' not in skip:
            self.node = NodeServerRunner(
                source=name + '.js',
                name=name, *args, **kwargs)
        if 'rust' not in skip:
            self.rust = RustServerRunner(
                source='../src/examples/server/{}/main.rs'.format(name),
                name=name, *args, **kwargs)

    def __iter__(self):
        if hasattr(self, 'go'):
            yield self.go
        if hasattr(self, 'node'):
            yield self.node
        if hasattr(self, 'rust'):
            yield self.rust

    def run_bencher_on_all(self, bencher, concurrency):
        '''
        Run each of the servers in the collection with the given bencher.

        :yields: (server runner, bench results)
        '''
        for server_runner in self:
            yield (server_runner,
                   bencher.start_server_and_bench(server_runner, concurrency))


class ServerBencher(object):

    TOOL = NotImplemented

    def start_server_and_bench(self, server_runner, concurrency):
        '''Start the (already compiled) server runner and run the tests.'''
        with tempmsg(
                'Running {} benchmark of {} {} server at concurrency {}...'
                .format(self.TOOL, server_runner.PLATFORM, server_runner.name,
                    concurrency)):
            with server_runner.activate():
                return self.bench(server_runner, concurrency)

    def bench(self, server_runner, concurrency):
        '''
        Actually run the tests. Requires the server to be started.

        Must be implemented by subclasses.
        '''
        raise NotImplementedError()


class ApacheBenchServerBencher(ServerBencher):

    TOOL = 'ab'

    def __init__(self, bin='ab'):
        self.bin = bin

    def bench(self, server_runner, concurrency):
        process = subprocess.Popen(
            (self.bin, '-n', '100000', '-c', str(concurrency),
                server_runner.root_url),
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        stdout, stderr = process.communicate()
        # Might fail here if it failed. Meh; no point catching it, let it fail.
        rps_line = next(line for line in stdout.split('\n')
                        if line.startswith('Requests per second:'))
        # Matches the 2323.84 part of:
        # Requests per second:    2323.84 [#/sec] (mean)
        return float(rps_line.split()[3])


def runners_benchers_cross_product(runners, benchers, concurrency):
    '''
    Run all combinations of runners (a :class:`ServerRunnerCollection`) and
    benchers (any iterable).

    :yields: (runner, bencher, results) tuples
    '''
    for bencher in benchers:
        for runner, results in runners.run_bencher_on_all(bencher, concurrency):
            yield runner, bencher, results


def main():
    ab = ApacheBenchServerBencher()
    #wrk = WrkServerBencher()
    benchers = [ab]

    for server_name in ('apache_fake',):
        runners = ServerRunnerCollection(
                name=server_name,
                build_dir='../build',
                hostname='127.0.0.1',
                port='8001')
        for runner in runners:
            with tempmsg('Compiling {} {} server...'
                    .format(runner.PLATFORM, runner.name)):
                runner.compile_server()

        for concurrency in (1, 2, 3, 4, 8):
            for runner, bencher, result in runners_benchers_cross_product(
                    runners, benchers, concurrency):
                print runner.PLATFORM, concurrency, bencher.TOOL, result


if __name__ == '__main__':
    main()
