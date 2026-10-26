export async function useAssignment(deps: any) { deps.other = function () { this.job(); }; await deps.job(); deps.other(); }
export async function useUpdate(deps: any) { deps.other++; await deps.job(); deps.other(); }
export async function useDelete(deps: any) { delete deps.other; await deps.job(); deps.other(); }
export async function useArray(deps: any) { [deps.other] = [function () { this.job(); }]; await deps.job(); deps.other(); }
export async function useObject(deps: any) { ({ other: deps.other } = { other: function () { this.job(); } }); await deps.job(); deps.other(); }
export async function useDefault(deps: any) { [deps.other = function () { this.job(); }] = []; await deps.job(); deps.other(); }
export async function useRest(deps: any) { [...deps.other] = []; await deps.job(); deps.other(); }
export async function useForOf(deps: any) { for (deps.other of [function () { this.job(); }]) {} await deps.job(); deps.other(); }
export async function useForIn(deps: any) { for (deps.other in {}) {} await deps.job(); deps.other(); }
export async function usePure(deps: any) { void deps.other; await deps.job(); deps.other(); }
// Nested targets and indirect invocation must not bypass receiver-shape checks.
export async function useNestedWrite(deps: any) { deps.other.call = function () { this.job(); }; await deps.job(); deps.other.call(deps); }
export async function useMemberChain(deps: any) { await deps.job(); deps.other.call(); }
export async function useNestedUpdate(deps: any) { deps.other.counter++; await deps.job(); }
