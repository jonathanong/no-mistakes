class Base {
  static run() {}
}

class Child extends Base {}

export const Alias = Child;

function localAlias() {
  // The same class identity also has a binding in a child lexical scope.
  const Local = Child;
  return Local;
}

localAlias();
