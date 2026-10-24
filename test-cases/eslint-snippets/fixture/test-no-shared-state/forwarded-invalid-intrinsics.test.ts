const helper = { call() {}, apply() {} };
// These ordinary methods are not registrations; normalization may have no callee.
(0, helper.call)();
(0, helper.apply)();
// Incomplete intrinsic invocations must not crash the rule either.
test.call.call();
test.apply.apply();
