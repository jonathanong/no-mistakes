// Calls `used` imported through the named barrel.
import { used } from "./named-barrel";

export const a = () => used();
