# Reference-bearing arm comparisons

The saved CommonJS module has two SQL builder references alongside twenty
scalar bindings, four arrow functions, and a captured callback. Its two
alternative arms mutate one builder while retaining the other. The configured
annotation rule selects this file with `unanalyzableSql: ignore`.

The unannotated retained builder must still produce findings both directly and
through the callback after the join. The work-count test expects four prefix
comparisons: two reference bindings in each arm. Pinned main performs 58 on the
same fixture. The source deliberately has no whole-object `module.exports`
assignment: that statement makes legacy extraction unsupported and would prevent
the comparison loop from being exercised.

The production guard runs before the observer and after-binding lookup. Function
capture environments remain independently reachable; the prefix comparison never
walked those environments. The per-file precise-record flag is also read once
per alternatives invocation. Work counts establish which comparisons are skipped;
consumer timing, observer overhead, and RSS remain separate measurement gates.
