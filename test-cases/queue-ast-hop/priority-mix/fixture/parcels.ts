import { parcels } from "./queues";

export function enqueueParcels(id: string) {
  return parcels.addBulk([
    { name: "storeParcel", data: { id } },
    { name: "holdParcel", data: { id }, opts: { priority: 0 } },
    { name: "rushParcel", data: { id }, opts: { priority: 7 } },
  ]);
}
