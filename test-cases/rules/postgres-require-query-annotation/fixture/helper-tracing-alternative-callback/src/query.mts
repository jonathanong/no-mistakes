import sql from 'sql-template-strings';
import { write } from '@app/db';

export function alternativeCallbackCanMutateCapture(condition: boolean) {
  const statement = sql`/* callback alternative */ SELECT 1`;
  const callback = condition ? (() => unknownMutation(statement)) : (() => 'unused');
  const ignored = unknownConsumer(callback);
  write(statement); // unanalyzable:alternative-callback-capture
}

export function returnedCallbackKeepsItsArmLocalFrame(condition: boolean) {
  const statement = sql`/* arm local callback */ SELECT 1`;
  function callbackFactory() {
    const local = statement;
    return () => unknownMutation(local);
  }
  const callback = condition ? callbackFactory() : (() => 'unused');
  const ignored = unknownConsumer(callback);
  write(statement); // unanalyzable:alternative-local-frame
}

export function conditionalSqlRemainsUnproven(condition: boolean) {
  write(condition ? '/* one */ SELECT 1' : '/* two */ SELECT 2'); // unanalyzable:conditional-sql
}

export function nestedArgumentsAppendMustNotRestoreEmptySql(condition: boolean) {
  function forward() {
    // The inline builder is reachable only through this private arguments slot.
    const ignored = condition ? arguments[0].append('SELECT 1') : '';
    write(arguments[0]); // unanalyzable:alternative-nested-builder
  }
  const ignored = forward(sql``);
}

export function alternativeAsyncCallbackKeepsCapturedFrame(condition: boolean) {
  const statement = sql`/* async callback frame */ SELECT 1`;
  async function factory() {
    const local = statement;
    return () => unknownMutation(local);
  }
  const callback = condition ? factory() : '';
  const ignored = unknownConsumer(callback);
  write(statement); // unanalyzable:alternative-promise-callback
}

export function alternativeArgumentsCallbacksKeepCapturedFrames(condition: boolean) {
  const statement = sql`/* arguments callback frame */ SELECT 1`;
  function factory() {
    // Empty aggregates must not keep speculative frames or break callback remapping.
    const empty = [];
    const local = statement;
    const callback = () => unknownMutation(local);
    function pack() { return arguments; }
    return pack(callback);
  }
  const callbacks = condition ? factory() : [];
  const ignored = unknownConsumer(callbacks);
  write(statement); // unanalyzable:alternative-arguments-callback
}

export async function alternativePromiseBuilderMutationIsMerged(condition: boolean) {
  async function wrap(value) { return value; }
  const pending = wrap(sql``);
  const ignored = condition ? (await pending).append('SELECT 1') : '';
  write(await pending); // unanalyzable:alternative-promise-builder
}
