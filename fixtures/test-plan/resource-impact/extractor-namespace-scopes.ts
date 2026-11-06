import { readFileSync } from 'node:fs';

readFileSync('before-namespace.json');
namespace Lexical {
  readFileSync('hidden-before-lexical.json');
  const readFileSync = (_path: string) => '';
  readFileSync('hidden-after-lexical.json');
}
readFileSync('after-lexical.json');
namespace Hoisted {
  readFileSync('hidden-before-var.json');
  if (true) {
    var readFileSync = (_path: string) => '';
  }
}
readFileSync('after-var.json');
namespace Outer {
  readFileSync('outer-before-nested.json');
  namespace Inner {
    var readFileSync = (_path: string) => '';
    readFileSync('hidden-inner.json');
  }
  readFileSync('outer-after-nested.json');
}
namespace Lexical {
  readFileSync('reopened-namespace.json');
}
readFileSync('after-namespaces.json');
