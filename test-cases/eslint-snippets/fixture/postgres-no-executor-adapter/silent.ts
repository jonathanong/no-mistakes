import { read } from "@example/db";

declare function run(exec: unknown, account?: string): unknown;
declare class Runner {
  constructor(exec?: unknown);
}

// Assignments and direct eval can replace a function declaration, so the
// named rewrite is not judged. A nested eval counts: strict eval can assign
// the outer function binding.
export function reassigned(account: string, other: unknown) {
  function adapter(statement: string) {
    return read(statement);
  }
  adapter = other;
  run(adapter, account);

  function closed(statement: string) {
    return read(statement);
  }
  function hide() {
    closed = other;
  }
  void hide;
  run(closed, account);

  function stable(statement: string) {
    return read(statement);
  }
  function dynamic() {
    eval("stable = null");
  }
  void dynamic;
  run(stable, account);
}

export function stored(
  account: string,
  flag: boolean,
  cond: boolean,
  name: string,
  client: { query(sql: string): unknown },
  other: unknown,
) {
  const options = { exec: (statement: string) => read(statement) };
  run(options, account);
  run(options.exec, account);
  const fns = [read];
  run(fns, account);
  run(fns[0], account);
  let rebound = read;
  run(rebound, account);
  run(read.bind, account);
  run(read?.bind(null), account);
  run(read.bind?.(null), account);
  run(((statement: string) => read(statement)).bind(null), account);
  run(read[name](null), account);
  run(read[`${name}`](null), account);
  run(read(account), account);
  run({ get exec() { return read; } }, account);
  run({ set exec(_value: unknown) {} }, account);
  class Box {
    #bind() {
      return read;
    }
    load() {
      run(this.#bind(null), account);
    }
  }
  void Box;
  const named = function name(statement: string) {
    run(name, account);
    return read(statement);
  };
  void named;
  function plain(statement: string) {
    return statement.trim();
  }
  run(plain, account);
  run(other, account);

  run(0 && read, account);
  run(false && read, account);
  run("" && read, account);
  run(0n && read, account);
  run(null && read, account);
  run(undefined && read, account);
  run(void 0 && read, account);
  run(NaN && read, account);
  run(!1 && read, account);
  run(`` && read, account);
  run(1 || read, account);
  run(true || read, account);
  run("x" || read, account);
  run(/a/ || read, account);
  run(`x` || read, account);
  run(0 ?? read, account);
  run(false ?? read, account);
  run("" ?? read, account);
  run(true ?? read, account);
  run(1 ?? read, account);
  run(NaN ?? read, account);
  run(!1 ?? read, account);
  run((0 as number) ?? read, account);
  run((0 as number) && read, account);
  run(typeof flag || read, account);
  run(typeof flag ?? read, account);
  run(flag === cond ?? read, account);
  run(`x${flag}` ?? read, account);
  run([] ?? read, account);
  run(new Runner() ?? read, account);
  run(!flag ?? read, account);
  run(+flag ?? read, account);
  // Updates are numbers, import() is a Promise, and delete is a boolean.
  // None of those are nullish, so `??` never selects `read`.
  run(flag++ ?? read, account);
  run(import("m") ?? read, account);
  const obj = { k: true };
  run(delete obj.k ?? read, account);
  run(new Runner() || read, account);
  run(read.bind(null, account), account);
  run(read.bind(null)(account), account);
  run(read && flag, account);
  run(read && client.query, account);
  new Runner(client.query);
  run({ exec: client.query }, account);
  run([client.query], account);
  run(client.query.bind(null), account);
  run(flag && client.query, account);
  run(client.query, account);
}
