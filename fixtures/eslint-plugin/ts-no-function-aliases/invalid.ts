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

async function asyncAlias(value: string) {
  return await original(value);
}

function original(value?: string) {
  return value;
}
