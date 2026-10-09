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
