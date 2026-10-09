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
