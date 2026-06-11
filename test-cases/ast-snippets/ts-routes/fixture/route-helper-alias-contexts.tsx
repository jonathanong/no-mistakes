import { entityHref } from './entity-href';
import { links } from './links';

const href = entityHref;
const router = useRouter();
router.push(href(entity));
router.replace(links.entityHref(entity));
router.replace(links?.entityHref);
try {
  router.push(entityHref(entity));
} catch (entityHref) {
  router.push(entityHref(entity));
}
