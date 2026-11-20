import * as parts from "./typeofTypeFragments";

type Columns = typeof parts;
type Column = typeof parts.TYPE_COLS;

export type WidgetColumns = Columns;
export type WidgetColumn = Column;
