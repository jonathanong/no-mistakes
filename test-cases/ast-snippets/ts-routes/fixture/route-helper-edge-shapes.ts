export function logicalHref(value: string | null): string {
  return value || `/logical/${value}`;
}
export function assertedHref(value: string): string {
  return (`/asserted/${value}` as string);
}
export function angleAssertedHref(value: string): string {
  return <string>`/angle/${value}`;
}
function objectHref({ id }: { id: string }): string {
  return `/object/${id}`;
}
export function wrappedObjectHref(entity: { id: string }): string {
  return objectHref(entity);
}
export function missingReturnHref(): string {
  return;
}
export function cappedHref(
  a = flag ? '/a' : '/b',
  b = flag ? '/c' : '/d',
  c = flag ? '/e' : '/f',
  d = flag ? '/g' : '/h',
  e = flag ? '/i' : '/j',
): string {
  return a + b + c + d + e;
}
export function branchedHref(entity: { id: string }, archived: boolean): string {
  if (archived) {
    return `/archive/${entity.id}`;
  }
  return `/active/${entity.id}`;
}
export function localBranchHref(entity: { id: string }, archived: boolean): string {
  if (archived) {
    const href = `/archive-local/${entity.id}`;
    return href;
  }
  return `/active-local/${entity.id}`;
}
export function deadReturnHref(entity: { id: string }): string {
  return `/dead-live/${entity.id}`;
  return `/dead-unreachable/${entity.id}`;
}
export function nestedStatementHref(entity: { id: string }, kind: string): string {
  if (kind === 'block') {
    {
      const { id: localId } = entity;
      const href = `/nested-block/${entity.id}`;
      return href;
    }
  } else if (kind === 'switch') {
    switch (kind) {
      case 'switch':
        return `/nested-switch/${entity.id}`;
      default:
        return `/nested-switch-default/${entity.id}`;
    }
  } else {
    try {
      return `/nested-try/${entity.id}`;
    } catch {
      return `/nested-catch/${entity.id}`;
    }
  }
  return `/nested-fallback/${entity.id}`;
}
export function nestedBranchEnvIsolationHref(entity: { id: string }, archived: boolean): string {
  let href = `/nested-branch/${entity.id}`;
  {
    if (archived) {
      href = `/nested-branch-archive/${entity.id}`;
    } else {
      return href;
    }
  }
  return href;
}
export function topLevelBlockReturnHref(entity: { id: string }): string {
  {
    return `/block-return/${entity.id}`;
  }
}
export function topLevelBlockAssignHref(entity: { id: string }): string {
  let href = `/block-assign/${entity.id}`;
  {
    href += '/details';
  }
  return href;
}
export function reassignedHref(entity: { id: string }, tab?: string): string {
  let href = `/users/${entity.id}`;
  if (tab) href += `/tabs/${tab}`;
  return href;
}
export function assignedHref(entity: { id: string }, tab?: string): string {
  let href = `/assigned/${entity.id}`;
  if (tab) href = `/assigned/${entity.id}/tabs/${tab}`;
  return href;
}
export function topLevelAssignedHref(entity: { id: string }): string {
  let href = `/top/${entity.id}`;
  href += '/edit';
  return href;
}
export function memberAssignmentIgnoredHref(entity: { id: string }): string {
  let href = `/member-assignment/${entity.id}`;
  target.href = `/ignored-member-assignment/${entity.id}`;
  return href;
}
export function destructuredLocalHref(entity: { id: string }): string {
  const { id } = entity;
  return `/destructured/${id}`;
}
export function reassignedBranchHref(entity: { id: string }, kind: string): string {
  let href = `/items/${entity.id}`;
  if (kind === 'a') href += '/a';
  else href += '/b';
  return href;
}
export function switchHref(entity: { id: string }, kind: 'user' | 'org'): string {
  switch (kind) {
    case 'user':
      return `/users/${entity.id}`;
    case 'org':
      return `/orgs/${entity.id}`;
    default:
      return `/unknown/${entity.id}`;
  }
}
export function reassignedSwitchHref(entity: { id: string }, kind: string): string {
  let href = `/switch/${entity.id}`;
  switch (kind) {
    case 'settings':
      href += '/settings';
      return href;
    case 'base':
      return href;
  }
  return href;
}
export function switchAllBranchesAssignedHref(entity: { id: string }, kind: string): string {
  let href = `/switch-all/${entity.id}`;
  switch (kind) {
    case 'settings':
      href += '/settings';
      break;
    default:
      href += '/default';
      break;
  }
  return href;
}
export function switchFallthroughHref(entity: { id: string }, kind: string): string {
  let href = `/fallthrough/${entity.id}`;
  switch (kind) {
    case 'a':
      href += '/a';
    case 'b':
      href += '/b';
      break;
  }
  return href;
}
export function emptySwitchHref(entity: { id: string }, kind: string): string {
  let href = `/empty-switch/${entity.id}`;
  switch (kind) {}
  return href;
}
export function tryHref(entity: { id: string }): string {
  try {
    return `/try/${entity.id}`;
  } catch {
    return `/fallback/${entity.id}`;
  }
}
export function tryFinallyHref(entity: { id: string }): string {
  try {
    let href = `/try-finally/${entity.id}`;
    return href;
  } finally {
    // Intentional fixture: a finally return overrides the try return.
    return `/finally/${entity.id}`;
  }
}
export function catchParamShadowHref(entity: { id: string }): string {
  let href = `/catch-shadow/${entity.id}`;
  try {
    throw new Error('boom');
  } catch (href) {
    return href;
  }
}
export const urlObjectHref = (entity: { id: string }) => ({
  pathname: `/object/${entity.id}`,
});
export const spreadObjectHref = (entity: { id: string }) => ({
  ...base,
  pathname: `/spread/${entity.id}`,
});
import { entityHref } from './entity-href';
export const composedHref = (entity: { id: string }) => `${entityHref(entity)}/settings`;
let noInit;
