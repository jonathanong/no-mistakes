Production root globs exclude test roots and allow only configured helpers for specific CI roots. One CI root is clean through a helper-directory exception; another still reports its allowed helper's forbidden dependency. All imports use relative paths, including the production → shared → test helper chain. The reverse test import must remain allowed.

Combined config-validation fixtures preserve literal bracket roots while keeping forbidden target patterns strict.
