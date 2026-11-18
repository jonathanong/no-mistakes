"use strict";

function rememberAlternatives(state, joined, left, right) {
  const tags = [joined, left, right];
  for (const tag of tags) {
    let aliases = state.iterationAliases.get(tag);
    if (!aliases) state.iterationAliases.set(tag, (aliases = new Set()));
    for (const other of tags) if (other !== tag) aliases.add(other);
  }
}

function unsafeReceiver(state, tag, aliases, seen = new Set()) {
  if (seen.has(tag)) return false;
  seen.add(tag);
  if (
    tag.iterationChanged ||
    tag.iterationUncertain ||
    (aliases && tag.iterationIndirectChanged) ||
    (tag.type === "fresh" && tag.escaped)
  )
    return true;
  if (
    tag.type === "union" &&
    tag.members.some((member) => unsafeReceiver(state, member, aliases, seen))
  )
    return true;
  if (aliases) {
    for (const member of state.iterationAliases.get(tag) || []) {
      if (unsafeReceiver(state, member, true, seen)) return true;
    }
  }
  return false;
}

module.exports = { rememberAlternatives, unsafeReceiver };
