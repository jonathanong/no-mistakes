const entityHref = (entity: { id: string }) => `/aliased/${entity.id}`;
export { entityHref as href };
