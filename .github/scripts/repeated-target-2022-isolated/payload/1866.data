import { read, sql } from '@example/db';
declare function opaque(...values: unknown[]): void;

function branch(flag: boolean) {
  if (flag) return ['left'];
  return ['right'];
}

export function earlierArgument(flag: boolean) {
  opaque([() => {
    const query = sql`/* transient/argument */ SELECT 1`;
    query.append('; DELETE FROM records');
    read(query);
  }], branch(flag));
}

export function earlierElement(flag: boolean) {
  opaque([[() => {
    const query = sql`/* transient/element */ SELECT 2`;
    query.append('; DELETE FROM records');
    read(query);
  }], branch(flag)]);
}

function privateCallback() {
  // Capture a private function: opaque callbacks deliberately weaken scalar captures.
  function append(query) {
    query.append('; DELETE FROM records');
  }
  return () => {
    const query = sql`/* transient/private */ SELECT 3`;
    append(query);
    read(query);
  };
}

export function temporaryClosure(flag: boolean) {
  opaque(privateCallback(), branch(flag));
}

export function callbackWorklist(flag: boolean) {
  opaque([() => {
    branch(flag);
    return [() => {
      const query = sql`/* transient/worklist */ SELECT 4`;
      query.append('; DELETE FROM records');
      read(query);
    }];
  }]);
}
