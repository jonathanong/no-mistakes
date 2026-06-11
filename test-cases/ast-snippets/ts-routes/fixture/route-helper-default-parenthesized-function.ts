export default (function (entity: { id: string }): string {
  return `/parenthesized-function/${entity.id}`;
});
