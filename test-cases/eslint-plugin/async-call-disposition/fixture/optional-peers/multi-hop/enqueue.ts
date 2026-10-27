import { dispatch, type Dependencies } from "./dispatch";

export async function submitForKey(key: string, dependencies: Dependencies = {}) {
  await dispatch({ ...dependencies, key });
}
