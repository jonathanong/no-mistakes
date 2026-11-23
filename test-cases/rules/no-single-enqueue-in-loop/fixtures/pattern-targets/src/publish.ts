import {
  enqueueWidget as publishWidget,
  enqueueBulkWidgets,
  enqueueBulkExplicit,
  enqueueLegacyWidget,
  persist as enqueueDecoy,
} from "./queue/widgets";
import * as widgets from "./queue/widgets";
import { enqueueWidget as enqueueOutside, pushWidget } from "./other/widgets";

export function publish(items: readonly string[]): void {
  for (const item of items) {
    publishWidget(item);
    widgets.enqueueWidget(item);
    enqueueBulkWidgets([item]);
    enqueueBulkExplicit(item);
    enqueueLegacyWidget(item);
    enqueueDecoy(item);
    enqueueOutside(item);
    pushWidget(item);
  }
}
