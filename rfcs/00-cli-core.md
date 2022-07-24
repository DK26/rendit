# CLI

<!--
Flowcharts Guide: https://github.com/mermaid-js/mermaid
Live Editor: https://mermaid.live
-->

## Graphs

### Core Flow

This document describes the core logical flow of the CLI, without getting into special options.

```mermaid
graph TD;
    Z(START)-->A{Template File Argument};
    A-->|Yes| B(Read File);
    A-->|No| F(Read STDIN);
    F-->Q;
    B-->Q;
    Q[Template Data]-->C{Context Argument};
    C-->|Yes| E(Load Context Argument);
    C-->|No| D(Load `default.ctx.json` Context);
    E-->W;
    D-->W;
    W[Context Data]-->G(Render Template Data with Context Data);
    G-->R;
    R[Render Result]-->|Output|H{Output Argument};
    H-->|No|L{Template File Argument};
    H-->|Yes|O(Output to file)
    O-->X;
    L-->|Yes|J(Output to `FILE NAME.rendered.FILE EXTENSION`);
    L-->|No|K(Print to STDOUT);
    K-->X;
    J-->X;
    X(END)
```  

---

### CLI States

```mermaid
stateDiagram
    [*] --> LoadedTemplate
    LoadedTemplate --> LoadedContext
    [*] --> Failed
    Failed --> [*]
    LoadedTemplate --> Failed
    LoadedContext --> RenderedTemplate
    LoadedContext --> Failed
    RenderedTemplate --> [*]
```
