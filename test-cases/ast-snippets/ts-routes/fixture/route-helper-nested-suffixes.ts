function getTopicTypeSlug(topicType: string): string {
  return topicType;
}
type Topic = { topic_type: string; id: string; slug?: string | null };
export function createTopicPathname(topic: Topic, suffix = ''): string {
  const idOrSlug = topic.slug ?? topic.id;
  return `/${getTopicTypeSlug(topic.topic_type)}/${idOrSlug}${suffix}`;
}
export function topicTagsHref(topic: Topic, tagType: string): string {
  return createTopicPathname(topic, `/tags/${tagType}`);
}
export function topicHref(topic: Topic, tab?: string): string {
  return createTopicPathname(topic, tab ? `/${tab}` : '');
}
