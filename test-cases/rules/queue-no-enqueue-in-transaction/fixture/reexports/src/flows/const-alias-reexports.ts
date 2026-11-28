import { beginTransaction } from "../db-named";
import { constAliasSend, constNsAliasSend, constObjSend } from "../jobs-named";

export function publishThroughConstAliases(rows: { id: string }[]) {
  return beginTransaction("write", async () => {
    rows.map(constAliasSend);
    rows.map(constNsAliasSend);
    rows.map(constObjSend.send);
  });
}
