# Fetch parse sharing

Target reachability and fetch extraction both need import facts from the page.
They must share one parsed-file cache so the page and its used target module are
each parsed exactly once.
