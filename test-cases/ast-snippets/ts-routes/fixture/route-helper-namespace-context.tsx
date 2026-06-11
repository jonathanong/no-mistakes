import * as links from './entity-href';
const { entityHref } = links;
const [notHelper] = links;
const destructuredLink = <Link href={entityHref(topic)} />;
const link = <Link href={links.topicHref(topic)} />;
