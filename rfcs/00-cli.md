# CLI

This document describes the logical flow of CLI features.

<!--
Flowcharts Guide: https://github.com/mermaid-js/mermaid
Live Editor: https://mermaid.live
-->
## Flowchart

```mermaid
graph TD;
    Z[Start]-->A;
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
    H{Template from File};
    H-->|Yes|J(Output to `.rendered.FILE EXTENSION>`);
    H-->|No|K(Print to STDOUT);
```  

TBD

