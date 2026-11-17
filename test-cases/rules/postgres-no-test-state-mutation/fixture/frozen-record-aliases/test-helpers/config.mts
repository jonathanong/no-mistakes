const configs = Object.freeze({
  first: Object.freeze({ column: 'first_id' }),
  second: Object.freeze({ column: 'second_id' }),
} as const);

export function getConfig(name: string) {
  if (!Object.hasOwn(configs, name)) throw new Error('unknown');
  return configs[name as keyof typeof configs];
}
