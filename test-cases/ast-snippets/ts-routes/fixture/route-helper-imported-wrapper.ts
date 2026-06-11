import { entityHref, topicHref } from './entity-href';

export const href = (entity: { id: string }) => entityHref(entity),
  topicLink = (topic: { slug: string }) => topicHref(topic),
  functionExpressionHref = function (entity: { id: string }) {
    return entityHref(entity);
  };

export function functionHref(entity: { id: string }) {
  return entityHref(entity);
}
