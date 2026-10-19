import { read, write } from "@example/db";

declare function run(exec: unknown, account?: string): unknown;
declare class Runner {
  constructor(...args: unknown[]);
}

export function edges(
  account: string,
  flag: boolean,
  cond: boolean,
  client: { flag: unknown; load(): unknown; query(sql: string): unknown },
) {
  function adapter(statement: string) {
    return read(statement);
  }
  const alias = adapter;
  run(alias, account);
  run(hoisted, account);
  function hoisted(statement: string) {
    return read(statement);
  }
  async function delayed(statement: string) {
    return read(statement);
  }
  run(delayed, account);
  function* generated(statement: string) {
    return read(statement);
  }
  run(generated, account);
  function logged(statement: string) {
    console.log(statement);
    return read(statement);
  }
  run(logged, account);

  new Runner(read, (statement: string) => write(statement));
  run({ exec: read, save: write }, account);
  run([read, , write], account);
  run({ exec(statement: string) { return read(statement); } }, account);
  run({ [account](statement: string) { return read(statement); } }, account);
  run({ ...client, exec: read }, account);
  const exec = read;
  run({ exec }, account);
  run(read.bind(read), account);
  new Runner(read.bind(read));
  run(read["bind"](null), account);
  run(read[`bind`](null), account);
  run((flag ? read : write).bind(null), account);
  run((flag ? read : other).bind(null), account);
  const bound = read.bind(null);
  run(bound, account);
  run(null ?? read, account);
  run(undefined ?? read, account);
  run(void 0 ?? read, account);
  run((null as unknown) ?? read, account);
  run(flag ?? read, account);
  run(0 || read, account);
  run(false || read, account);
  run("" || read, account);
  run(flag || read, account);
  run(1 && read, account);
  run(true && read, account);
  run("x" && read, account);
  run(!0 && read, account);
  run(!!1 && read, account);
  run(1n && read, account);
  run(/a/ && read, account);
  run(`x` && read, account);
  run(`${flag}` && read, account);
  run((1 as number) && read, account);
  run(flag && (cond ? read : write), account);
  run(client.query ?? read, account);
  run(read ?? client.query, account);
  run(client.flag && read, account);
  run(client.load() && read, account);
  run(client?.flag && read, account);
  run(client?.flag ?? read, account);
  run(!flag && read, account);
  run(typeof flag && read, account);
  run(new Runner() && read, account);
  run(flag === cond && read, account);
  run(flag === cond || read, account);
  run(+account.length && read, account);
  run((account, flag) && read, account);
  run(read && write, account);
  run(read || write, account);
  run((flag ? read : other) ?? write, account);

  function overload(statement: string): unknown;
  function overload(statement: string) {
    return read(statement);
  }
  run(overload, account);
}

export function shadowed(undefined: unknown, NaN: unknown, account: string) {
  run(undefined ?? read, account);
  run(NaN && read, account);
}
