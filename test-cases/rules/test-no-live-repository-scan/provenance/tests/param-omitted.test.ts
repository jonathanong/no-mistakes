// Reported: the call omits the argument, so the default directory is scanned.
import { syncOmitted } from "../src/param-omitted";

syncOmitted("admin");
