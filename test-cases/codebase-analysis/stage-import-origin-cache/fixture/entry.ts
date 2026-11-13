import { first, second, mutable, scalar } from './stages';
for (const stage of first) stage.run();
for (const stage of second) stage.run();
