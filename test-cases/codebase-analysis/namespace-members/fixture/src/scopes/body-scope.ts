// A local declared in a namespace body, `const` or hoisted `var`, is bound only
// inside that body. The constructions after it still name the imported
// namespaces.
import { BodyErrors, VarErrors } from "./body-scope-lib";

namespace Helpers {
  const BodyErrors = {};
  var VarErrors = {};
  export const probe = [BodyErrors, VarErrors];
}

export const built = new BodyErrors.Built();
export const varBuilt = new VarErrors.VarBuilt();
