class Service {
  static run() {}
}

class InheritedBase {
  static run() {}
}
class InheritedChild extends InheritedBase {}

const ExpressionService = class {
  static run() {}
  instance() {}
};

const NamedExpressionService = class InternalExpressionService {
  constructor() {}
  static run() {}
  static self() {
    InternalExpressionService.run();
  }
};
const NamedExpressionAlias = NamedExpressionService;
const namedExpressionRun = NamedExpressionService.run;

let ReassignedExpression = class ReassignedInternal {
  static run() {}
};
ReassignedExpression = ExpressionService;

function namedExpressionNest() {
  const NestedPublic = class NestedInternal {
    constructor() {}
    static run() {}
  };
  new NestedPublic();
  NestedPublic.run();
}

function api() {
  function run() {}
}

Service.run();
InheritedChild.run();
ExpressionService.run();
ExpressionService.instance();
new NamedExpressionService();
NamedExpressionService.run();
new NamedExpressionAlias();
NamedExpressionAlias.run();
namedExpressionRun();
ReassignedExpression.run();
api.run();

// The internal display name repeats, but each outward binding must retain its
// own class and static-method CallableId.
{
  const FirstPublic = class Internal {
    static run() {}
  };
  new FirstPublic();
  FirstPublic.run();
}
{
  const SecondPublic = class Internal {
    static run() {}
  };
  new SecondPublic();
  SecondPublic.run();
}
