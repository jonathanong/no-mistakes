import { query } from '@example/db';

type Kind = 'orders' | 'refunds';

export function compound(kind: Kind, suffix: string) {
  kind += suffix;
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function arrayAssignment(kind: Kind, choices: [Kind]) {
  [kind] = choices;
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function objectAssignment(kind: Kind, choice: { kind: Kind }) {
  ({ kind } = choice);
  return query(`SELECT 1 FROM ${kind}_accounts`);
}

export function closureAssignment(kind: Kind, replacement: string) {
  const replace = () => {
    kind = replacement as Kind;
  };
  replace();
  return query(`SELECT 1 FROM ${kind}_accounts`);
}
