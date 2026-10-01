// A local in a namespace body, `const` or hoisted `var`, is bound only inside
// that body, so the constructions after it still name the imported namespaces.
import { BodyScope, BodyVar } from "./namespace-body-scope-lib";

namespace Helpers {
  const BodyScope = {};
  var BodyVar = {};
  export const probe = [BodyScope, BodyVar];
}

export const built = new BodyScope.ScopeBuilt();
export const varBuilt = new BodyVar.VarBuilt();
