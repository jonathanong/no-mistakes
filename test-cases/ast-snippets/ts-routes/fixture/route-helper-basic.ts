export function entityHref(entity: { id: string }, kind: string): string {
  return `/prefix/${entity.id}/suffix/${kind}`;
}
