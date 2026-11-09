import { unusedControl as constantControl } from "../targets/constant.mts";
import { unusedControl as functionControl } from "../targets/function-constant.mts";
import { unusedControl as directControl } from "../targets/url-direct.mts";
import { unusedControl as hrefControl } from "../targets/url-href.mts";
import { unusedControl as stringControl } from "../targets/url-string.mts";
import { unusedControl as pathnameControl } from "../targets/url-pathname.mts";

void [constantControl, functionControl, directControl, hrefControl, stringControl, pathnameControl];
