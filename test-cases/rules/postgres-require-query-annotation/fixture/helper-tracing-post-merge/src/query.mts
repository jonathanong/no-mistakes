import sql from 'sql-template-strings';
import { write } from '@app/db';

export function supportedTagMutatesCapture() {
  const statement = sql`/* before supported untrusted tag */ SELECT 1`;
  const mutator = () => { unknownMutation(statement); return 'ignored'; };
  const ignored = mutator`x`;
  write(statement); // unanalyzable:supported-untrusted-tag-capture
}
// var shares the parameter binding; its initializer executes after the first sink.
function parameterBeforeInitializedVar(statement: unknown) {
  write(statement); // known:parameter-before-var-initializer
  var statement = other;
  write(statement); // unanalyzable:parameter-after-var-initializer
}
export function invokeParameterVar() {
  parameterBeforeInitializedVar(sql`/* actual parameter */ SELECT 1`);
}
function mutateThroughArguments() {
  unknownMutation(arguments[0]);
}
export function builderPassedThroughArguments() {
  const statement = sql`/* before arguments mutation */ SELECT 1`;
  const ignored = mutateThroughArguments(statement);
  write(statement); // unanalyzable:implicit-arguments-mutation
}
function initializedVarExecutesInOrder(statement: unknown) {
  write(statement); // known:parameter-before-static-var
  var statement = sql`SELECT 2`;
  write(statement); // finding:parameter-after-static-var
}
export function invokeStaticParameterVar() {
  initializedVarExecutesInOrder(sql`/* initial parameter */ SELECT 1`);
}
function mutateThroughArrowArguments() {
  const arrow = () => unknownMutation(arguments[0]);
  return arrow();
}
export function arrowInheritsArguments() {
  const statement = sql`/* before lexical arguments mutation */ SELECT 1`;
  const ignored = mutateThroughArrowArguments(statement);
  write(statement); // unanalyzable:arrow-lexical-arguments
}
function nestedRegularOwnsArguments(statement: unknown) {
  function inner() { unknownMutation(arguments[0]); }
  const ignored = inner('opaque');
  write(statement); // known:nested-regular-own-arguments
}
export function invokeRegularOwnArguments() {
  nestedRegularOwnsArguments(sql`/* outer argument untouched */ SELECT 1`);
}

export function onlyTheSelectedArgumentCanMutate() {
  const statement = sql`/* selected second argument */ SELECT 1`;
  function mutateSecond() { unknownMutation(arguments[1]); }
  const ignored = mutateSecond(statement, 'other');
  write(statement); // known:unselected-argument
}
export function stringIndexAndArgumentAliasesSelectOneValue() {
  const statement = sql`/* string index */ SELECT 1`;
  function mutateSecond() {
    const values = arguments;
    unknownMutation(values['1']);
  }
  const ignored = mutateSecond(statement, 'other');
  write(statement); // known:string-index
}
export function unknownArgumentIndexRemainsConservative(index: number) {
  const statement = sql`/* unknown index */ SELECT 1`;
  function mutate() { unknownMutation(arguments[index]); }
  const ignored = mutate(statement, 'other');
  write(statement); // unanalyzable:dynamic-index
}
export function missingArgumentIndexHasNoBuilderAlias() {
  const statement = sql`/* out of range */ SELECT 1`;
  function mutate() { unknownMutation(arguments[9]); }
  const ignored = mutate(statement);
  write(statement); // known:missing-index
}
export function ordinaryAggregateMembersStayConservative() {
  const statement = sql`/* ordinary aggregate */ SELECT 1`;
  const values = { other: statement };
  const ignored = unknownMutation(values[0]);
  write(statement); // unanalyzable:ordinary-aggregate
}

export function noncanonicalAndFractionalIndicesRemainConservative() {
  const first = sql`/* noncanonical index */ SELECT 1`;
  const second = sql`/* fractional index */ SELECT 1`;
  function mutateFirst() { unknownMutation(arguments['01']); }
  function mutateSecond() { unknownMutation(arguments[0.5]); }
  const ignoredFirst = mutateFirst(first);
  const ignoredSecond = mutateSecond(second);
  write(first); // unanalyzable:noncanonical-index
  write(second); // unanalyzable:fractional-index
}
export function stringZeroRetainsTheSelectedBuilderIdentity() {
  const statement = sql`/* string zero */ SELECT 1`;
  function mutateFirst() { unknownMutation(arguments['0']); }
  const ignored = mutateFirst(statement, 'other');
  write(statement); // unanalyzable:string-zero
}

export function spreadsCannotProveArgumentSlotPositions() {
  function forward() {
    write(arguments[1]); // unanalyzable:spread-argument-slots
  }
  const ignored = forward(...['SELECT 1', 'SELECT 2'], '/* annotated */ SELECT 3');
}
export function escapingArgumentObjectCanReplaceImmutableSlots() {
  function forward() {
    const before = arguments[0];
    const slots = arguments;
    const ignored = unknownMutation(slots);
    write(before); // known:copied-immutable-slot
    write(arguments[0]); // unanalyzable:escaped-immutable-slot
  }
  const ignored = forward('/* original slot */ SELECT 1');
}
export function arrowArgumentEscapeSharesTheContainerIdentity() {
  function forward() {
    const ignored = (() => unknownMutation(arguments))();
    write(arguments[0]); // unanalyzable:arrow-argument-escape
  }
  const ignored = forward('/* arrow slot */ SELECT 1');
}
export function possibleBranchEscapeTaintsArgumentSlots(flag: boolean) {
  function forward() {
    const ignored = flag ? unknownMutation(arguments) : undefined;
    write(arguments[0]); // unanalyzable:branch-argument-escape
  }
  const ignored = forward('/* branch slot */ SELECT 1');
}

export function deletingArgumentSlotCannotRetainItsPreviousValue() {
  function forward() {
    const ignored = delete arguments[0];
    write(arguments[0]); // unanalyzable:deleted-slot
  }
  const ignored = forward('/* deleted slot */ SELECT 1');
}
export function replacingArgumentSlotCannotRetainItsPreviousValue() {
  function forward() {
    const ignored = (arguments[0] = 'SELECT 1');
    write(arguments[0]); // unanalyzable:replaced-slot
  }
  const ignored = forward('/* replaced slot */ SELECT 1');
}

export function deletingANonReferenceDoesNotMutateArgumentSlots() {
  function forward() {
    const ignored = delete 'unused';
    write(arguments[0]); // known:delete-non-reference
  }
  const ignored = forward('/* untouched slot */ SELECT 1');
}

export function appendedBuilderRefreshesItsArgumentSlot() {
  function forward(statement) {
    const wrapper = { statement };
    statement.append('/* appended alias */ SELECT 1');
    write(arguments[0]); // known:appended-argument-slot
  }
  const ignored = forward(sql``);
}
export async function appendedBuilderRefreshesPromiseAliases() {
  const statement = sql``;
  const pending = (async () => statement)();
  statement.append('/* pending alias */ SELECT 1');
  write(await pending); // known:appended-promise-alias
}
export function deletingSlotPreservesTheReferencedBuilder() {
  const statement = sql`/* retained builder */ SELECT 1`;
  function remove(parameter) {
    const ignored = delete arguments[0];
    write(parameter); // known:deleted-slot-parameter-alias
  }
  const ignored = remove(statement);
  write(statement); // known:deleted-slot-outer-alias
}
export function deletingSlotStillEvaluatesComputedKeyEffects() {
  const statement = sql`/* key effects */ SELECT 1`;
  function remove(parameter) {
    const ignored = delete arguments[unknownMutation(parameter)];
  }
  const ignored = remove(statement);
  write(statement); // unanalyzable:deleted-slot-key-effect
}

export function standaloneSlotDeletionPreservesReferencedBuilder() {
  const statement = sql`/* standalone removal */ SELECT 1`;
  function remove(parameter) {
    delete arguments[0];
    write(parameter); // known:standalone-delete-parameter
    return parameter;
  }
  const returned = remove(statement);
  write(returned); // known:standalone-delete-returned-alias
}
export function standaloneSlotDeletionKeepsComputedKeyEffects() {
  const statement = sql`/* standalone key */ SELECT 1`;
  function remove(parameter) {
    delete arguments[unknownMutation(parameter)];
    write(parameter); // unanalyzable:standalone-delete-key-effect
  }
  const ignored = remove(statement);
}
