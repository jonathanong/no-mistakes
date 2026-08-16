import { Feature0 } from './features/feature-0';
import { Feature1 } from './features/feature-1';
import { Feature2 } from './features/feature-2';
import { Feature3 } from './features/feature-3';
import { Feature4 } from './features/feature-4';
import { Feature5 } from './features/feature-5';
import { coreFn0 } from '@fixture/core';
import { dataRecord0 } from '@fixture/data';
import { clientCall0 } from '@fixture/http';
export async function GraphGatesEntry() {
  await clientCall0();
  return (
    <main>
      {coreFn0()}
      {dataRecord0.id}
      <Feature0 />
      <Feature1 />
      <Feature2 />
      <Feature3 />
      <Feature4 />
      <Feature5 />
    </main>
  );
}
