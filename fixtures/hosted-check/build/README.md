These text files model compiler outputs without compiling Rust. Tests copy this
saved tree to a temporary directory and supply a mocked process runner. The
placeholder package files must disappear before the real-source cargo call;
`libnum_cpus-saved.rlib` must survive. The `.rlib` files contain text markers.
