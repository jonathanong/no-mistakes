The sixteen intentionally identical helper files create 64 distinct `(file, type)`
cache keys. Each key follows sixteen explicit type re-export hops; thirty-two
receiver calls per helper then exercise cache hits. Keep the separate files and
repeated calls so the fixture retains both cold-key concurrency and repeated-key
pressure. The graph test compares one-thread and eight-thread output.
