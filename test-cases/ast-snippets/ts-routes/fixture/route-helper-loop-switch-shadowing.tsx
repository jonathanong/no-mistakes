import { entityHref } from './entity-href';

const router = useRouter();
for (const entityHref of hrefs) {
  router.push(entityHref(row));
}
for (const row of rows) {
  router.push(entityHref(row));
}
switch (kind) {
  case 'local':
    const entityHref = localHref;
    router.push(entityHref(row));
    break;
  case 'remote':
    router.push(entityHref(row));
    break;
}
