// An import edge alone must not make this module a reachable call source.
setTimeout(() => {}, 1);
