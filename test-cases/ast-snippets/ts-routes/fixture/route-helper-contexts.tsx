import { aHref, bHref } from './entity-href';

const concatLink = <Link href={aHref(a) + bHref(b)} />;
const templateLink = <Link href={`${aHref(a)}${bHref(b)}`} />;
const link = <Link href={flag ? aHref(a) : bHref(b)} />;
const objectLink = <Link href={{ pathname: flag ? aHref(a) : bHref(b) }} />;
const router = useRouter();
router.push(flag ? aHref(a) : bHref(b));
router.replace(aHref(a) || bHref(b));
