# .NET missing dependency baseline

Synthetic project file, central package manifest, and NuGet lockfile. There is
no comparison baseline. The three paths differ, and they are the same missing
baseline, so the plan reports one `dependency-baseline-missing` entry.

`after/Directory.Packages.props` is the same manifest with `Version="1.0.1"`.
Tests copy it over the committed file when they need a readable baseline.
