// Same-line expressions must remain distinguishable by their canonical source offsets.
export function select(flag: boolean, T1: any, T2: any, reg: any, name: string) { new (flag ? T1 : T2)(); new reg[name](); }
