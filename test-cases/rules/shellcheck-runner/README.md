
The saved `fixture/unavailable` scripts exercise missing ShellCheck execution
through report, advise, and ignore policies, suppression, and empty-selection
controls. Tests clear only a child process PATH; the parent environment stays
available to other suites. `native-check.cjs` compares compiled CJS/ESM,
batched check, and CLI reports against the same scripts and configs.
