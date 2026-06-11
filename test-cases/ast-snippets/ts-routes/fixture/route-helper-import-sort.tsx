import { betaHref } from './b';
import { alphaHref } from './a';
import { entityHref } from './entity-href';

const left = <><Link href={betaHref(entity)} /><Link href={alphaHref(entity)} /></>;
const right = <Link href={entityHref(entity)} />;
