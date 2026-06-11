import { entityHref } from './entity-href';

const hashLink = <Link href={entityHref(entity) + '#reviews'} />;
const queryLink = <Link href={`${entityHref(entity)}?tab=details`} />;
const prefixLink = <Link href={'/prefix' + entityHref(entity)} />;
const optionalLink = <Link href={entityHref?.(entity)} />;
const loose = entityHref(entity) + '#ignored';
