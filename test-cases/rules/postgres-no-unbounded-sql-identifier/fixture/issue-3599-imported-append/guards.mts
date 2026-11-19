import sql from 'sql-template-strings';
import { read } from './db.mjs';
import { customFields, escapedFields, overriddenFields, mixedFields, openFields, copyFields, calledFields, copyCalled, appliedFields, copyApplied } from './guard-projection.mjs';
declare function consume(value: unknown): void;
export function customDestination(destination: { append(value: unknown): void }, flag: boolean) {
  destination.append(customFields);
  const statement = sql`/* customDestination */ SELECT `;
  statement.append(customFields);
  statement.append(sql` FROM painted_widgets w`);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function escapedDestination(flag: boolean) {
  const destination = sql``;
  consume(destination);
  destination.append(escapedFields);
  const statement = sql`/* escapedDestination */ SELECT `;
  statement.append(escapedFields);
  statement.append(sql` FROM painted_widgets w`);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function overriddenDestination(flag: boolean) {
  const destination = sql``;
  destination.append = value => destination;
  destination.append(overriddenFields);
  const statement = sql`/* overriddenDestination */ SELECT `;
  statement.append(overriddenFields);
  statement.append(sql` FROM painted_widgets w`);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function mixedDestinations(destination: { append(value: unknown): unknown }, flag: boolean) {
  copyFields(sql``);
  copyFields(destination);
  const statement = sql`/* mixedDestinations */ SELECT `;
  statement.append(mixedFields);
  statement.append(sql` FROM painted_widgets w`);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function importedOpenInterpolation(flag: boolean) {
  const statement = sql`/* importedOpenInterpolation */ SELECT * FROM `;
  statement.append(openFields);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function forwardedCall(destination: { append(value: unknown): unknown }, flag: boolean) {
  copyCalled.call(sql`SELECT `, destination);
  const statement = sql`/* forwardedCall */ SELECT `;
  statement.append(calledFields);
  statement.append(sql` FROM painted_widgets w`);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
export function forwardedApply(destination: { append(value: unknown): unknown }, flag: boolean) {
  copyApplied.apply(sql`SELECT `, [destination]);
  const statement = sql`/* forwardedApply */ SELECT `;
  statement.append(appliedFields);
  statement.append(sql` FROM painted_widgets w`);
  if (flag) statement.append(sql` LIMIT ${10}`);
  return read(statement);
}
