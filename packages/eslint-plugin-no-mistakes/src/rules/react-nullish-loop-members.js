"use strict";

function mergeMembers(target, source) {
  for (const [name, props] of source) {
    const prior = target.get(name);
    target.set(name, prior ? new Set([...prior, ...props]) : props);
  }
}

function copyLoopMembers(loop, parent) {
  if (!loop || !parent) return;
  mergeMembers(parent.objectTypes, loop.objectTypes);
  mergeMembers(parent.unresolvedMembers, loop.unresolvedMembers);
}

function notePair(primary, aliases, binding, value) {
  if (!value) return;
  primary.set(binding, value);
  aliases.set(binding, value);
}

module.exports = { copyLoopMembers, notePair };
