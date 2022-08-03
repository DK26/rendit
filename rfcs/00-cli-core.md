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
    TSC{Succeeded};
    TSC ==> |Yes| Q;
    TSC .-> |No| X;
    Q[Template Data];
    Q --> C;
    C{'context' Argument};
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
    SC{Succeeded};
    SC ==> |Yes| W;
    SC .-> |No| X;
    W[Context Data];
    W --> G;
    G(Render Template with Context);
    G --> GSC;
    GSC{Succeeded}
    GSC ==> |Yes| R;
    GSC .-> |No| X;
    R[Rendered Template];
    R --> |Output| STDOUT;
    R --> |Output| STDERR;
    STDOUT{'stdout' switch};
    STDOUT ==> |Yes| PRINT_STDOUT
    PRINT_STDOUT(Print to STDOUT);
    STDERR{'stderr' switch};
    STDERR ==> |Yes| PRINT_STDERR
    PRINT_STDERR(Print to STDERR);
    R --> |Output| H;
    H{'output' Argument};
    H --> |No| L;
    L{Has Template File Argument};
    H ==> |Yes| O;
    O(Output to Argument file);
    O ==> SC_OUTPUT;
    L ==> |Yes| J;
    J(Output to `FILE NAME.rendered.FILE EXTENSION`);
    J ==> SC_OUTPUT;
    SC_OUTPUT{Succeeded};
    SC_OUTPUT ==>|Yes| FILE;
    SC_OUTPUT .->|No| X;
    FILE[Rendered File] ==> OPEN_SWITCH;
    OPEN_SWITCH{'open' switch};
    OPEN_SWITCH ==> |Yes| OPEN_FILE;
    OPEN_SWITCH .-> |No| X;
    OPEN_FILE(Open File for Preview);
    OPEN_FILE .-> X;
    L --> |No| STDOUT2;
    STDOUT2{'stdout' switch}
    STDOUT2 --> |No| K;
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
    X(END);
```  
