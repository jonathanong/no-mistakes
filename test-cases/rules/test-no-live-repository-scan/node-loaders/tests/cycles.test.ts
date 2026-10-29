// These invalid runtime initializations must terminate the static provenance walk.
const first = second;
const second = first;
first("node:fs");
second("glob");
