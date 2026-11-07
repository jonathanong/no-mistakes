import { mkdtempSync, readdirSync } from 'node:fs'
function branch() {
 if (flag) action()
 return mkdtempSync('prefix')
}
function tail() {
 const root = mkdtempSync('prefix')
 action(root)
}
function accepted() {
 const root = mkdtempSync('prefix')
 action(root)
 return root
}
readdirSync(branch())
readdirSync(tail())
readdirSync(accepted())
