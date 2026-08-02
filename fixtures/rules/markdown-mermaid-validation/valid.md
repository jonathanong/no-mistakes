# Valid diagrams

```mermaid
flowchart TD
A --> B
```

``` MERMAID title=Sequence
sequenceDiagram
Alice->>Bob: Hello
```

````mermaid
stateDiagram-v2
[*] --> Idle
````

<!-- markdownlint-disable-next-line MD048 -->
> ~~~mermaid
> flowchart LR
> Quote --> Works
> ~~~

- ```mermaid
  flowchart TD
  List --> Works
  ```
