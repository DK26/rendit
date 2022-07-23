# CLI

This document describes the logical flow of CLI features.

<!--
Flowcharts Guide: https://github.com/mermaid-js/mermaid
Live Editor: https://mermaid.live
-->
## Flowchart

```mermaid
graph TD;
    Z[START]-->A;
    A{Template File Argument};
    A--> |Yes| B(Read File);
    A--> |No| F(Read STDIN);
    F-->C;
    B-->C;
    C{Context Argument};
    C--> |Yes| E(Use Argument Context);
    C--> |No| D(Use `default.ctx.json` for Context);
    E-->G;
    D-->G;
    G(Render Contents);
    G-->H;
    H{Output Argument};
    H-->|No|L;
    H-->|Yes|O(Output to file)
    O-->X;
    L{Template File Argument};
    L-->|Yes|J(Output to `FILE NAME.rendered.FILE EXTENSION`);
    L-->|No|K(Print to STDOUT);
    K-->X;
    J-->X;
    X[END]
```  

TBD
