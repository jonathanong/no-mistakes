const configs = Object.freeze({
  a: Object.freeze({ table: 'retained_a', column: 'a_id' }),
  b: Object.freeze({ table: 'retained_b', column: 'b_id' }),
});

export function getConfig(name: string) {
  if (!Object.hasOwn(configs, name)) throw new Error(`unknown ${name}`);
  return configs[name as keyof typeof configs];
}
