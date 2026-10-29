import * as declaredNamespace from '../db/index';
import * as foreignNamespace from '../other/index';
import foreignDefault from '../other/index';
import type * as erasedNamespace from '../db/index';
import type erasedDefault from '../db/index';
import shadowedDefault from '../db/index';
import * as shadowedNamespace from '../db/index';

export async function outside() {
  const otherTx = await declaredNamespace.unrelatedFactory();
  await otherTx('DROP TABLE app.factory_wrong_export'); // OUTSIDE: undeclared export
  const foreignTx = await foreignNamespace.beginTransaction();
  await foreignTx('DROP TABLE app.factory_foreign_namespace'); // OUTSIDE: foreign module
  const foreignDefaultTx = await foreignDefault();
  await foreignDefaultTx('DROP TABLE app.factory_foreign_default'); // OUTSIDE: foreign default
  const erasedTx = await erasedNamespace.beginTransaction();
  await erasedTx('DROP TABLE app.factory_erased_namespace'); // OUTSIDE: erased namespace
  const erasedDefaultTx = await erasedDefault();
  await erasedDefaultTx('DROP TABLE app.factory_erased_default'); // OUTSIDE: erased default
}

export async function shadowed(
  shadowedDefault: () => Promise<(sql: string) => unknown>,
  shadowedNamespace: { beginTransaction: () => Promise<(sql: string) => unknown> },
) {
  const defaultTx = await shadowedDefault();
  await defaultTx('DROP TABLE app.factory_shadow_default'); // OUTSIDE: shadowed default
  const namespaceTx = await shadowedNamespace.beginTransaction();
  await namespaceTx('DROP TABLE app.factory_shadow_namespace'); // OUTSIDE: shadowed namespace
}
