import { entityHref } from './entity-href';
import * as links from './links';

function Row({ entityHref, links }) {
  return (
    <>
      <Link href={entityHref(row)} />
      <Link href={links.entityHref(row)} />
    </>
  );
}

const link = <Link href={entityHref(entity)} />;
const namespaceLink = <Link href={links.entityHref(entity)} />;
const router = useRouter();
{
  const entityHref = localBuilder;
  router.push(entityHref(row));
}
