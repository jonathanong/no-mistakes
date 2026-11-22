import sql from "@example/sql";
import {aliasColumns} from "@example/aliases/columns";
import {workspaceColumns} from "@example/fragments";

const suffix = sql`, extra_id`;
aliasColumns.append(suffix);
workspaceColumns.append(suffix);
