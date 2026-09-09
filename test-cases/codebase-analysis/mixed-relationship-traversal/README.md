# Mixed relationship traversal fixture

The entry imports a module that calls another module at top level. A traversal
filtered to both `import` and `call` must follow that combined path.
The reverse traversal from the called module must likewise cross the call edge
and then the import edge back to the entry module.
