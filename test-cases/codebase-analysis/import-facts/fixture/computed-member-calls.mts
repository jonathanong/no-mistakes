import * as playwright from 'playwright';

page['waitForTimeout'](1_000);
playwright['test']();
(playwright as typeof playwright)['test']();
