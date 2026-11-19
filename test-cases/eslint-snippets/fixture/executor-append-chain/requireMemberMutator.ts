const parts = require("./requireMemberControls");

export function extend(extra: unknown) {
  parts.REQUIRE_MEMBER.append(extra);
}
