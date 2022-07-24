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
    Z(START)-->A{Has Template File Argument};
    A-->|Yes| B(Read File);
    A-->|No| F(Read STDIN);
    F-->Q;
    B-->Q;
    Q[Template Data]-->C{Has Context Argument};
    C-->|No| T;
    T{Has Template File Argument};
    T-->|Yes| Y;
    Y(Load Context `FILE NAME.ctx.json`)-->W;
    C-->|Yes| E(Load Context Argument);
    T-->|No| D(Load Context `default.ctx.json`);
    E-->W;
    D-->W;
    W[Context Data]-->G(Render Template with Context);
    G-->R;
    R[Render Result]-->|Output|H{Has Output Argument};
    H-->|No| L{Has Template File Argument};
    H-->|Yes| O(Output to file)
    O-->X;
    L-->|Yes| J(Output to `FILE NAME.rendered.FILE EXTENSION`);
    L-->|No| K(Print to STDOUT);
    K-->P{CTRL + C};
    U(Read STDIN)-->I(Render Template with Context)
    I--> M[Render Result]
    M-->|Output| K
    P-->|Yes| X
    P-->|No| U
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
