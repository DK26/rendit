# rendit (🚧WIP🚧)

![render-template](assets/logo.gif)

CLI tool for rendering templates with local JSON files as context. Supporting multiple template engines.

## 🚧WIP Status🚧

For now, you can download an [alpha release](https://github.com/DK26/rendit/releases) version and play around with it:

[Download Alpha v0.1.3](https://github.com/DK26/rendit/releases/download/v0.1.3-alpha/rendit-alpha-v.0.1.3-windows-i686.7z)

### All Releases

<https://github.com/DK26/rendit/releases> 

## Features

* Renders template input either from file or standard input 🚧
* Uses either the default or the specified context JSON file for rendering ✅
* Automatically detects the right rendering engine with either a special starting comment (magic comment) `<!--template ENGINE_NAME>`  or by the template's file extension  ✅
* Allows you to manually decide which engine to use out of the supported engine list: `--engine-list` ✅

* Allows you to automatically open the rendered output file for preview with a default software ✅
* Allows you to watch constantly for changes in both the template and its context file ✅
* Built with pipeline support 🚧
* Supports splitting output between STDOUT, STDERR and output file ✅

## Commandline Usage

<!--Examples TBD-->


<details>
<summary>Usage Help (click to expand)</summary>

```text
USAGE:
    rendit [OPTIONS] [TEMPLATE FILE]

ARGS:
    <TEMPLATE FILE>
            The template file to render.

            This requires either the `<TEMPLATE NAME>.ctx.json` or the `default.ctx.json` context
            files to be present in the template's directory.

            [Example]

            .
            ├── my_template.ctx.json
            └── my_template.html

            If `my_template.ctx.json` is missing, `default.ctx.json` is automatically loaded
            instead.

            .
            ├── default.ctx.json
            └── my_template.html

            This behavior can be overridden by assigning the context file directly, using the
            `--context <CONTEXT FILE>` option.

            [Output]

            Unless using the `--output <OUTPUT FILE>` option,
            providing `<TEMPLATE FILE>`, produces a `<TEMPLATE NAME>.rendered.<EXTENSION>` file by
            default.

            .
            ├── my_template.html
            └── my_template.rendered.html

            By NOT providing `<TEMPLATE FILE>`, STDIN mode is activated. Template data stream will
            be expected to come from STDIN, printing the results to STDOUT instead of writing to
            file.

OPTIONS:
    -c, --context <CONTEXT FILE>
            Override default loading of the context file with the specified context file.

    -o, --output <OUTPUT FILE>
            Override default output file path with the specified file path.

        --stdout
            Print rendered result to STDOUT.

        --stderr
            Print rendered result to STDERR.

    -O, --open
            Open the rendered output file with a default software.

    -w, --watch <SECONDS>
            Constantly render changes in the template with the context file every 2 seconds by
            default.

    -e, --engine <ENGINE NAME>
            Force rendering with the specified render engine.
            Use only when there is no magic comment or a template file extension available.

        --engine-list
            Print supported engine list for the `--engine <ENGINE NAME>` option.

    -x, --extension <EXTENSION>
            Force the underlying template engine to treat the template data as if it is a file of
            the given extension. This may affect escaping of special characters.

    -v, --verbose
            Set the level of verbosity.

            `-v` sets logging level to INFO

            `-vv` sets logging level to DEBUG

            `-vvv` sets logging level to TRACE

            WARNING: Effects CLI / STDOUT output.
            Use the `--output` switch if you wish to commit the rendered output to file.
            Use the `--stderr` switch to avoid including the logger messages in the final output.

    -h, --help
            Print help information

    -V, --version
            Print version information
```

</details>

## Supported Template Engines

| Name       | Short / File Extension  | Version | Guide / Manual / Tutorial                                     |  
| ---------- | ----------------------- | ------- | ------------------------------------------------------------- |
| Tera       | `tera`                  | v1.17.0 | <https://tera.netlify.app/docs/#templates>                    |
| Handlebars | `hbs`                   | v4.3.3  | <https://handlebarsjs.com/guide/>                             |
| Liquid     | `liq`                   | v0.26.0 | <https://github.com/Shopify/liquid/wiki/Liquid-for-Designers> |

## Template Examples

<details>
<summary>Tera (click to expand)</summary>

* Guide: <https://tera.netlify.app/docs/#templates>  
* Version: **v1.17.0**
* Repository: <https://github.com/Keats/tera>
* Alternatives: `Jinja2`, `Django`, `Liquid`, `Twig`
  
A highly advanced, capable and secure by default; rendering engine that follows the OWASP Top 10 guidelines.
A good alternative choice if you are used to template engines such as `Jinja2`, `Django`, `Liquid` or `Twig`. Originated in the Rust programming language.  

```html
<HTML>
    🚧WIP🚧
</HTML>
```

</details>

<details>
<summary>Handlebars (click to expand)</summary>

* Guide: <https://handlebarsjs.com/guide/>  
* Version: **v4.3.3**
* Repository: <https://github.com/sunng87/handlebars-rust>
* Alternatives: `Mustache`
  
A highly popular rendering engine that has been implemented across many programming languages. Considered to be somewhat more limited in features compared to the other engines. Originated in the Javascript programming language.

```html
<HTML>
    🚧WIP🚧
</HTML>
```

</details>

<details>
<summary>Liquid (click to expand)</summary>

* Guide: <https://github.com/Shopify/liquid/wiki/Liquid-for-Designers>  
* Version: **v0.26.0**
* Repository: <https://github.com/cobalt-org/liquid-rust>
* Alternatives: `smarty`
  
A highly advanced, capable and senior rendering engine, offering some optional security capabilities. A good alternative choice if you are used to the `smarty` template engine. Originated in the Ruby programming language.

```html
<HTML>
    🚧WIP🚧
</HTML>
```

</details>
