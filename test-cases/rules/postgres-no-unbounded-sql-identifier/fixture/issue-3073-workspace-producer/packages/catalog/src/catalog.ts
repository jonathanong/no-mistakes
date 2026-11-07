export interface Relation {
  subject: string;
  predicate: string;
  object: string;
  table_name: string;
}

export const RELATIONS: Relation[] = [];
for (const subject of ["topic", "account"]) {
  for (const predicate of ["parent"]) {
    for (const object of ["topic"]) {
      RELATIONS.push({
        subject,
        predicate,
        object,
        table_name: ["rel", subject, predicate, object].join("_"),
      });
    }
  }
}

export function getRelationOrThrow(label: Pick<Relation, "subject" | "predicate" | "object">) {
  const relation = RELATIONS.find(value =>
    value.subject === label.subject && value.predicate === label.predicate && value.object === label.object);
  if (relation) return relation;
  throw new Error("missing relation");
}

export function getRelationTableNameOrThrow(label: Parameters<typeof getRelationOrThrow>[0]): string {
  return getRelationOrThrow(label).table_name;
}
