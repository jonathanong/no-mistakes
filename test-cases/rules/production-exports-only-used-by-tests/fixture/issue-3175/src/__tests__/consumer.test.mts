import { constantOnly, unusedControl as constantControl } from "../targets/constant.mts";
import { functionConstantOnly, unusedControl as functionControl } from "../targets/function-constant.mts";
import { directOnly, unusedControl as directControl } from "../targets/url-direct.mts";
import { hrefOnly, unusedControl as hrefControl } from "../targets/url-href.mts";
import { stringOnly, unusedControl as stringControl } from "../targets/url-string.mts";
import { pathnameOnly, unusedControl as pathnameControl } from "../targets/url-pathname.mts";

void [constantControl, functionControl, directControl, hrefControl, stringControl, pathnameControl];
void [constantOnly, functionConstantOnly, directOnly, hrefOnly, stringOnly, pathnameOnly];
