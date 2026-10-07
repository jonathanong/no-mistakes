type NodeRequire = (specifier: string) => unknown;
(require as NodeRequire)("./cast");
require!("./nonnull");
(require)("./parenthesis");
(require satisfies NodeRequire)("./satisfies");
(<NodeRequire>require)("./assertion");
require<string>("./instantiation");
