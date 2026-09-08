# Call traversal fixture

This fixture covers cross-file call diamonds, a multi-node cycle, callable
roots with only global calls, and an unknown computed call that must not become
a guessed graph edge. It also locks namespace-import resolution, unresolved
runtime imports, local and external re-exports, private local exports,
lexical-parent callable lookup, and tagged-template calls. Named local and
imported tags resolve normally, while computed and dynamically produced tags
stay unresolved.
