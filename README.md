# A Parallel Mandelbrot Set Plotter

This program plots the Mandelbrot set and writes it out as a PNG file. It uses Rust's concurrency primitives to distribute the work across eight threads.

Different commits show different implementation strategies:

- Commit 218c0d6 splits the plotting area up into eight bands, and assigns one
  thread to each. This often makes poor use of the threads, because some bands
  take significantly longer than others to complete: once a fast thread
  completes its band, it twiddles its thumbs waiting for its less fortunate
  bretheren to finish their tasks.

- Commit ca5f89b gets an almost perfect linear speedup from its threads. It
  splits the plotting area up into many more bands, and then has threads draw
  bands from a common pool until the pool is empty. When a thread finishes one
  band, it goes back for more work. Since the bands still take different amounts
  of time to render, the problem cited above still occurs, but on a much smaller
  scale.

- Commit ad81b19 uses Rust's atomic types to implement a lock-free iterator
  type, and uses that to dole out bands from the pool instead of a
  mutex-protected count. On Linux, this is no faster than the mutex-based
  version, which isn't too surprising: on Linux, locking and unlocking an
  uncontended mutex *is* simply a pair of atomic operations.
