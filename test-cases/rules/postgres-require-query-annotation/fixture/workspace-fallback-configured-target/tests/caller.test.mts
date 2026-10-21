// Although excluded, this caller reaches the configured executor package/subpath.
import { execute } from '@fixture/db/read';
execute("SELECT excluded_caller_context");
