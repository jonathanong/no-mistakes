import { enqueueBare } from "./factory";
import { packedViaConst, widgets } from "./queues";

enqueueBare(widgets, "bare-factory-missing");
enqueueBare(packedViaConst, "bare-factory-covered");
