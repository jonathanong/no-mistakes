const configs = Object.freeze({
  a: Object.freeze({ column: 'a_id' }),
  b: Object.freeze({ column: 'b_id' }),
  c: Object.freeze({ column: 'c_id' }),
  d: Object.freeze({ column: 'd_id' }),
  e: Object.freeze({ column: 'e_id' }),
  f: Object.freeze({ column: 'f_id' }),
  g: Object.freeze({ column: 'g_id' }),
  h: Object.freeze({ column: 'h_id' }),
  i: Object.freeze({ column: 'i_id' }),
  j: Object.freeze({ column: 'j_id' }),
  k: Object.freeze({ column: 'k_id' }),
  l: Object.freeze({ column: 'l_id' }),
  m: Object.freeze({ column: 'm_id' }),
  n: Object.freeze({ column: 'n_id' }),
  o: Object.freeze({ column: 'o_id' }),
  p: Object.freeze({ column: 'p_id' }),
  q: Object.freeze({ column: 'q_id' }),
});

export function getConfig(name: string) {
  if (typeof name !== 'string' || !Object.hasOwn(configs, name)) {
    throw new Error(`unknown ${name}`);
  }
  return configs[name as keyof typeof configs];
}

export function getEarly(name: string) {
  if (typeof name !== 'string') {
    return;
  }
  if (!Object.hasOwn(configs, name)) {
    return undefined;
  }
  return configs[name as keyof typeof configs];
}
