import { readdirSync } from 'node:fs';
function direct(args) {
  const [command, directory] = args;
  if (command === 'scan') readdirSync(directory, 'direct');
}
function reversed(args) {
  const [command, directory] = args;
  if ('scan' === command) readdirSync(directory, 'reversed');
}
function hole(args) {
  const [, command, directory] = args;
  if (command === 'scan') readdirSync(directory, 'hole');
}
function numeric(args) {
  const [mode, directory] = args;
  if (mode === 2) readdirSync(directory, 'numeric');
}
function conjunction(args) {
  const [command, ...rest] = args;
  if (command === 'scan' && rest.length === 1) readdirSync(rest[0], 'conjunction');
}
function restForwarded(args) {
  const [command, ...rest] = args;
  if (rest.length === 1) readdirSync(rest[0]!, 'rest-forwarded');
}
function loose(args) {
  const [command, directory] = args;
  if (command == 'scan') readdirSync(directory, 'loose');
}
function disjunction(args) {
  const [command, directory] = args;
  if (command === 'scan' || command === 'list') readdirSync(directory, 'disjunction');
}
function alternate(args) {
  const [command, directory] = args;
  if (command === 'scan') {} else readdirSync(directory, 'alternate');
}
function late(args) {
  setup();
  const [command, directory] = args;
  if (command === 'scan') readdirSync(directory, 'late');
}
function multiple(args) {
  const [command, directory] = args, other = 1;
  if (command === 'scan') readdirSync(directory, 'multiple');
}
function defaulted(args = []) {
  const [command, directory] = args;
  if (command === 'scan') readdirSync(directory, 'defaulted');
}
function extra(args, option) {
  const [command, directory] = args;
  if (command === 'scan') readdirSync(directory, 'extra');
}
function reused(args) {
  const [command, directory] = args;
  consume(args);
  if (command === 'scan') readdirSync(directory, 'reused');
}
function nested(args) {
  const [[command], directory] = args;
  if (command === 'scan') readdirSync(directory, 'nested');
}
function written(args) {
  let [command, directory] = args;
  command = 'scan';
  if (command === 'scan') readdirSync(directory, 'written');
}
function restEscaped(args) {
  const [command, ...rest] = args;
  consume(rest);
  if (rest.length === 1) readdirSync(rest[0], 'rest-escaped');
}
function restWrongComparison(args) {
  const [command, ...rest] = args;
  if (rest.length > 0) readdirSync(rest[0], 'rest-wrong-comparison');
}
function directMember(args) {
  if (args.length === 1) readdirSync(args[0], 'direct-member');
}
function boolean(args) {
  const [command, directory] = args;
  if (command === true) readdirSync(directory, 'boolean');
}
