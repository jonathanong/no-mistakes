// This ignored shadow must not become visible merely because Button.tsx is an explicit root.
import { IgnoredButton } from "../ignored-explicit/Button";

test("ignored shadow", () => {
  void IgnoredButton;
});
