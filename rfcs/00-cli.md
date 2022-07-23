# CLI

This document describes the logical flow of CLI features.

<!--Flowcharts Guide: https://github.com/mermaid-js/mermaid-->
## Flowchart

```mermaid
graph TD;
    A{Template File Argument};
    A--> |Yes| B[1];
    A--> |No| F[Read STDIN];
    C-->F;
    C{Context Argument};
    C--> |Yes| E[Use Argument Context];
    C--> |No| D[Use `default.ctx.json` for Context];
```  

TBD

