export function linkedHref(entity: { id: string }): string {
  return `/linked/${entity.id}`;
}
