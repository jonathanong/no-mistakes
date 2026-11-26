import { use } from "react";

use(Promise.resolve(1));
use(Promise.resolve(1) as Promise<number>);
(use as typeof use)(Promise.resolve(1));
(use<number>)(Promise.resolve(1));
