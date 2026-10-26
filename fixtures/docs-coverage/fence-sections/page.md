# Fence and heading boundary regression

~~~markdown
## Valid example
wrong section inside a fence
~~~

## Valid example notes
wrong section with an arbitrary suffix

## Valid example and explanation

~~~~markdown
## Inside tilde fence
````
## Different fence character cannot close
~~~
## Shorter fence cannot close
~~~~ trailing text
## Closing fence cannot have trailing text
~~~~~

## Counterexample, with fix

````markdown
## Inside backtick fence
~~~~
## Different fence character still cannot close
```
## Shorter backtick fence cannot close
`````

## Options
outside sections
