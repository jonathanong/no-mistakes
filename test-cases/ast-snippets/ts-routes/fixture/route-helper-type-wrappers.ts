import { entityHref } from './entity-href';

const router = useRouter();
router.push(entityHref(entity)!);
router.replace((entityHref(entity) satisfies string));
router.prefetch(<string>entityHref(entity));
router.push(getLinks().entityHref(entity));
