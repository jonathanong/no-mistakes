from pathlib import Path
import json, hashlib
root=Path(__file__).resolve().parent
root.mkdir(parents=True,exist_ok=True)
for i in range(272):
    ext='tsx' if i%4==0 else 'ts'
    lines=['import { step } from "./shared";',f'export function work{i}(value: number) {{']
    lines += [f'  value = step(value, {j});' for j in range(1040)]
    lines += ['  return value;', '}']
    if ext=='tsx':
        lines += [f'export function View{i}() {{ return <section data-index="{i}">{{work{i}(0)}}</section>; }}']
    (root/f'unit-{i:03}.{ext}').write_text('\n'.join(lines)+'\n')
(root/'shared.ts').write_text('export function step(value: number, amount: number) { return value + amount; }\n')
(root/'entry.ts').write_text('import { work0 } from "./unit-000";\nexport function run() { work0(0); setTimeout(() => {}, 1); }\nrun();\n')
config='version: 2\nrules:\n'
for i in range(13):
    config += f'''  - name: boundary-{i}
    rule: forbidden-calls
    scope: repository
    include: ["**/*.ts", "**/*.tsx"]
    options:
      roots: [{{glob: ["**/*.ts", "**/*.tsx"]}}]
      traversal: transitive
      unknownCalls: ignore
      targets: [{{global: {"setTimeout" if i==0 else "forbiddenApi"+str(i)}}}]
'''
(root/'on.yml').write_text(config)
(root/'off.yml').write_text('version: 2\nrules: []\n')
sources=sorted([*root.glob('*.ts'),*root.glob('*.tsx')])
census={'files':len(sources),'bytes':sum(p.stat().st_size for p in sources),'lines':sum(len(p.read_bytes().splitlines()) for p in sources),'jsxFiles':len(list(root.glob('*.tsx'))),'inventoryHash':hashlib.sha256(b''.join(p.name.encode()+b'\0'+p.read_bytes() for p in sources)).hexdigest()}
print(json.dumps(census,indent=2))
