# Directory roots

Recursive directory roots honor rule filters, path components, and overlapping roots.
The nested source reaches its forbidden module through a helper outside the origin scope.

Relative and absolute literal file/directory roots intersect the prepared file
universe, including a known-empty universe; existing files do not widen it.
