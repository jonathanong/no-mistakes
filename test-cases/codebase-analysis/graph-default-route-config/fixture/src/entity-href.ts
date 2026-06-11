export function entityHref(entity: { id: string }, kind: string): string {
  return `/prefix/${entity.id}/suffix/${kind}`;
}

export function crawlerHref(crawler: { id: string }): string {
  return `/crawler/${crawler.id}/edit`;
}
