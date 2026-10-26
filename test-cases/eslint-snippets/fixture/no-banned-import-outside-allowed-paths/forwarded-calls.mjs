import { invalidateAll as clear } from "@acme/rate-limit";
clear();
(0, clear)();
clear.call(null);
clear["call"](null);
clear.apply(null, []);
clear["apply"](null, []);
const alias = clear;
alias();
alias.call(null);
const key = "call";
clear[key](null);
function shadow(clear) { clear.call(null); }
import * as namespace from "@acme/rate-limit";
// Literal exported names call/apply must win over forwarding interpretation.
namespace.call();
namespace.apply();
// no-mistakes-disable-next-line no-banned-import-outside-allowed-paths -- protocol fixture
clear.call(null);
clear.apply(null, []); // no-mistakes-disable-line no-banned-import-outside-allowed-paths -- protocol fixture
// Unknown forwarded argument lists stay conservative.
clear.call(...args);
clear.apply(null, args);
clear.apply(null, [,]);
clear.apply(null, [...args]);
function shadowAlias(alias) { (0, alias)(); }
alias?.();
clear[`call`](null);
clear[`apply`](null, []);
namespace.call.call(null);
namespace.apply.apply(null, []);
clear.call.call(clear, null);
clear.apply.call(clear, null, []);
clear.call.apply(clear, [null]);
clear.apply.apply(clear, [null, []]);
// Intrinsic forwarding with an unrelated explicit receiver is not clear().
clear.call.call(other, null);
clear.apply.call(other, null, []);
clear.call.apply(other, [null]);
clear.apply.apply(other, [null, []]);
(0, clear.call).call(clear, null);
(0, clear.call)(null);
