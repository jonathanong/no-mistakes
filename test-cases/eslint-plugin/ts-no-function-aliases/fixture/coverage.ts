const functionExpression = function (value: string) {
  return original(value);
};

function recursive(value: string) {
  return recursive(value);
}

const recursiveArrow = (value: string) => recursiveArrow(value);

const defaultParam = (value = "x") => original(value);

const nonIdentifierParam = ({ value }: { value: string }) => original(value);

const chain = (value: string) => original?.(value);

const assertion = (value: string) => original!(value);

declare function declared(value: string): string;

function notExpression(value: string) {
  if (condition) {
    original(value);
  }
}

if (condition) {
  original("x");
}

function original(value: string) {
  return value;
}

const condition = true;

export {
  assertion,
  chain,
  defaultParam,
  functionExpression,
  nonIdentifierParam,
  notExpression,
  recursive,
  recursiveArrow,
  declared,
};
