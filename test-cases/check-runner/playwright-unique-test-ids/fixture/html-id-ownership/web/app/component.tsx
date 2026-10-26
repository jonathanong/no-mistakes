import { Widget } from '@fixture/ui';

const componentId = 'component-derived-id';

export function Component() {
  return <Widget htmlId={componentId} />; // EXPECT_COMPONENT
}
