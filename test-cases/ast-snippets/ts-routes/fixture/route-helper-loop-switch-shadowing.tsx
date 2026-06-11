import { entityHref } from './entity-href';

const router = useRouter();
for (const entityHref of hrefs) {
  router.push(entityHref(row));
}
for (const row of rows) {
  router.push(entityHref(row));
}
for (const router of routers) {
  router.push(entityHref(row));
}
for (const href = entityHref; ready; step()) {
  router.push(href(row));
}
switch (kind) {
  case 'local': {
    const entityHref = localHref;
    router.push(entityHref(row));
    break;
  }
  case 'router-local': {
    const router = localRouter;
    router.push(entityHref(row));
    break;
  }
  case 'remote':
    router.push(entityHref(row));
    break;
}
