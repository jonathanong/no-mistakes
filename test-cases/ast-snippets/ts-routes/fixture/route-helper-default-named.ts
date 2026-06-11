export default function entityHref(entity: { id: string }): string {
  return `/entities/${entity.id}`;
}
