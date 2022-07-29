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
    Z(START);
    Z --> A;
    A{Has Template File Argument};
    A ==> |Yes| B;
    B(Load Template File Argument);
    A --> |No| F;
    F(Read STDIN);
    F --> TSC;
    B --> TSC;
    TSC{Read Successfully};
    TSC ==> |Yes| Q;
    TSC .-> |No| X;
    Q[Template Data];
    Q --> C;
    C{Has Context Argument};
    C ==> |Yes| E;
    C --> |No| T;
    T{Has Template File Argument};
    T ==> |Yes| YT;
    T --> |No| D;
    YT{`FILE NAME.ctx.json` File Exists};
    YT ==> |Yes| Y;
    YT --> |No| D;
    Y(Load Context `FILE NAME.ctx.json`);
    Y --> SC;
    E(Load Context Argument);
    D(Load Context `default.ctx.json`);
    E --> SC;
    D --> SC;
    SC{Read Successfully};
    SC ==> |Yes| W;
    SC .-> |No| X;
    W[Context Data];
    W --> G;
    G(Render Template with Context);
    G --> R;
    R[Rendered Template];
    R --> |Output| H;
    H{Has Output Argument};
    H --> |No| L;
    L{Has Template File Argument};
    H ==> |Yes| O;
    O(Output to Argument file);
    O .-> X;
    L ==> |Yes| J;
    J(Output to `FILE NAME.rendered.FILE EXTENSION`);
    L --> |No| K;
    K(Print to STDOUT);
    K --> P;
    P{CTRL + C};
    U(Read STDIN);
    U --> I;
    I(Render Template with Context);
    I --> M;
    M[Rendered Template];
    M --> |Output| K;
    P .-> |Yes| X;
    P --> |No| U;
    J .-> X;
    X(END);
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
