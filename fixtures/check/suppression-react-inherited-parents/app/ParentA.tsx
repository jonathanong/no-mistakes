// no-mistakes-disable-file assert-no-fetch: this parent intentionally inherits a suppressed child fetch
import Child from './Child';

export default async function ParentA() {
  return <Child />;
}
