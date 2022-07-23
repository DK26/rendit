# CLI

This document describes the core logical flow of the CLI, without getting into special options.

## Core Flow

<!--
Flowcharts Guide: https://github.com/mermaid-js/mermaid
Live Editor: https://mermaid.live
-->

```mermaid
graph TD;
    Z(START)-->A;
    A{Template File Argument};
    A--> |Yes| B(Read File);
    A--> |No| F(Read STDIN);
    F-->Q;
    B-->Q;
    Q[Template Data]-->C;
    C{Context Argument};
    C--> |Yes| E(Load Context Argument);
    C--> |No| D(Load `default.ctx.json` Context);
    E-->W;
    D-->W;
    W[Context Data]-->G;
    G(Render Template Data with Context Data);
    G-->R;
    R[Render Result]-->|Output|H;
    H{Output Argument};
    H-->|No|L;
    H-->|Yes|O(Output to file)
    O-->X;
    L{Template File Argument};
    L-->|Yes|J(Output to `FILE NAME.rendered.FILE EXTENSION`);
    L-->|No|K(Print to STDOUT);
    K-->X;
    J-->X;
    X(END)
```  
