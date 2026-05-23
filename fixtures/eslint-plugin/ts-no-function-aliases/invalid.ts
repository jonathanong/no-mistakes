export function noArgs() {
  return original();
}

export function withArg(value: string) {
  return original(value);
}

export function sideEffect(value: string) {
  original(value);
}

export const arrow = (value: string) => original(value);

export const asyncArrow = async (value: string) => await original(value);

export const restArrow = (...args: string[]) => original(...args);

exports.propertyArrow = (value: string) => original(value);

module.exports.propertyFunction = function (value: string) {
  return original(value);
};

api.property = (value: string) => original(value);

exports.sameName = (value: string) => sameName(value);

exports["computed"] = (value: string) => original(value);

async function asyncAlias(value: string) {
  return await original(value);
}

function outerAlias() {
  function nested(value: string) {
    return original(value);
  }
  return nested("x");
}

function original(value?: string) {
  return value;
}
