import { read, sql } from '@example/db';
export const query = sql`SELECT 1`;
export function branchCallback(flag: boolean, input: unknown[]) {
  // Capture the supplied module's exported builder.
  if (flag) {
    const callback = () => {
      for (const value of input) appendKnown(query);
      read(query);
    };
    callback();
  }
}

export function falseBranchCallback(input: unknown[]) {
  // A discarded branch cannot mutate the exported builder.
  if (false) {
    const callback = () => {
      for (const value of input) appendKnown(query);
      read(query);
    };
    callback();
  }
  read(query);
}

function appendKnown(target: unknown) {
  target.append(sql`; DELETE FROM records`);
}

export function neverInvokedCallback(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => {
      for (const value of input) appendKnown(query);
      read(query);
    };
  }
}

export function hoistedBranchCallback(flag: boolean, input: unknown[]) {
  if (flag) {
    function callback() {
      for (const value of input) appendKnown(query);
      read(query);
    }
    callback();
  }
}

export function aliasCallback(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); }; const alias = callback;
    alias();
  }
}

export function recordCallback(flag: boolean, input: unknown[]) {
  if (flag) {
    const holder = { run: () => { for (const value of input) appendKnown(query); read(query); } };
    holder.run();
  }
}

export function arrayCallback(flag: boolean, input: unknown[]) {
  if (flag) {
    const holder = [() => { for (const value of input) appendKnown(query); read(query); }];
    holder[0]();
  }
}

export function neverInvokedContainers(flag: boolean, input: unknown[]) {
  if (flag) {
    const record = { run: () => { for (const value of input) appendKnown(query); read(query); }, data: 1 };
    const array = [() => { for (const value of input) appendKnown(query); read(query); }, 1];
  }
}

export function shadowedAlias(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    {
      const callback = 0;
      const alias = callback;
      alias();
    }
  }
}

export function aliasBeforeDeclaration(flag: boolean, input: unknown[]) {
  if (flag) {
    // The TDZ alias cannot invoke the later declaration.
    const alias = callback;
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    alias();
  }
}

export function deepUnusedContainer(flag: boolean) {
  if (flag) {
    // Inert metadata respects the same evaluation budget as ordinary expressions.
    const unused = { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: () => { read(query); }, ignored: 1 } } } } } } } } } } } } } } } } } } } } } } } } };
  }
}

export function deepInvokedContainer(flag: boolean, input: unknown[]) {
  if (flag) {
    const holder = { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: { child: () => { for (const value of input) appendKnown(query); read(query); } } } } } } } } } } } } } } } } } } } } } } } } } };
    holder.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child();
  }
}

export function deepInvokedArray(flag: boolean, input: unknown[]) {
  if (flag) {
    const holder = [[[[[[[[[[[[[[[[[[[[[[[[[() => { for (const value of input) appendKnown(query); read(query); }]]]]]]]]]]]]]]]]]]]]]]]]];
    holder[0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0]();
  }
}

export function budgetFailureRoots(flag: boolean) {
  if (flag) {
    external.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child();
    read(query);
  }
  if (flag) {
    ({}).child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child.child();
    read(query);
  }
}

export function cyclicBudgetContainer(flag: boolean, input: unknown[]) {
  if (flag) {
    const callback = () => { for (const value of input) appendKnown(query); read(query); };
    const holder = [[[[[[[[[[[[[[[[[[[[[[[[[callback]]]]]]]]]]]]]]]]]]]]]]]]];
    // A budget failure may expose a cyclic supplied container.
    holder.push(holder);
    holder[0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0][0]();
  }
}

export function outerAlias(flag: boolean, input: unknown[]) {
  const callback = () => { for (const value of input) appendKnown(query); read(query); };
  if (flag) { const alias = callback; alias(); }
}

export function outerHoisted(flag: boolean, input: unknown[]) {
  function callback() { for (const value of input) appendKnown(query); read(query); }
  if (flag) { const alias = callback; alias(); }
}

export function outerRecord(flag: boolean, input: unknown[]) {
  const holder = { run: () => { for (const value of input) appendKnown(query); read(query); } };
  if (flag) { const trigger = 0; holder.run(); }
}

export function outerArray(flag: boolean, input: unknown[]) {
  const holder = [() => { for (const value of input) appendKnown(query); read(query); }];
  if (flag) { const trigger = 0; holder[0](); }
}

export function outerDeletedSlot(flag: boolean, input: unknown[]) {
  const holder = [0, () => { for (const value of input) appendKnown(query); read(query); }];
  delete holder[0];
  if (flag) { const trigger = 0; holder[1](); }
}
