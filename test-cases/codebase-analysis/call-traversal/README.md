# Call traversal fixture

This fixture covers cross-file call diamonds, a multi-node cycle, callable
roots with only global calls, and an unknown computed call that must not become
a guessed graph edge. It also locks namespace-import resolution, unresolved
runtime imports, private local exports, and lexical-parent callable lookup.
