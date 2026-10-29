import { beginTransaction as namedFactory } from '../db/index';
import * as factoryNamespace from '../db/index';
import defaultFactory from '../db/index';

export async function run() {
  const namedTx = await namedFactory();
  await namedTx('DROP TABLE app.factory_named_control'); // named import is selected

  const namespaceTx = await factoryNamespace.beginTransaction();
  await namespaceTx('DROP TABLE app.factory_namespace'); // ADVISE: namespace result is not a named-import proof

  const computedTx = await factoryNamespace['beginTransaction']();
  await computedTx('DROP TABLE app.factory_computed'); // ADVISE: literal namespace key

  const defaultTx = await defaultFactory();
  await defaultTx('DROP TABLE app.factory_default'); // ADVISE: default result is not a named-import proof
}
