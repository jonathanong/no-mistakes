import { entityHref } from './entity-href';

const assertedHref = <typeof entityHref>entityHref;
const router = useRouter();
router.push(assertedHref(entity));
