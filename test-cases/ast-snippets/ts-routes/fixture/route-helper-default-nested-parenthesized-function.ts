export default ((function (entity: { id: string }): string {
  return `/nested-parenthesized-function/${entity.id}`;
}));
