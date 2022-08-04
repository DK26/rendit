![render-template](assets/logo.gif)

CLI tool for rendering templates with local JSON files as context. Supporting multiple template engines.

## Features

* Rendering template input either from file or standard input
* Built with piping in mind
* Uses either the default or the specified context JSON file for rendering
* Supports splitting output between STDOUT, STDERR and output file
* Can open the rendered output file for preview with a default software
* Watch constantly for changes in both the template and its context file
* Detects the right rendering engine with either a special starting comment (magic comment) `<!--template ENGINE_NAME>`  or by the template's file extension  
* Can manually decide which engine to use out of the supported engine list: `--engine-list`

## Commandline Usage

<!--Examples TBD-->


<details>
<summary>Usage Help</summary>

```text
USAGE:
    rendit [OPTIONS] [TEMPLATE FILE]

ARGS:
    <TEMPLATE FILE>
            The template file to render.

            This requires either the `<TEMPLATE NAME>.ctx.json` or the `default.ctx.json` context
            files, to be present in the same directory.

            [Example]

            For the template file `my_template.html`, the automatic context file would be
            `my_template.ctx.json` of the same directory.

            If `my_template.ctx.json` is missing, `default.ctx.json` is automatically loaded
            instead.

            This behavior can be overridden by assigning the context file manually when using the
            `--context` option.

            [Output]

            Providing `<TEMPLATE FILE>` automatically produces `<TEMPLATE
            NAME>.rendered.<extension>` unless using the `--output` option.

            By NOT providing `<TEMPLATE FILE>`, STDIN mode is activated and will be waiting for
            template data stream in STDIN, printing results to STDOUT instead of writing to file.

OPTIONS:
    -c, --context <CONTEXT FILE>
            Override automatic loading of the context file with the specified context file

    -e, --engine <ENGINE NAME>
            Force rendering with the specified render engine. Use only when there is no magic
            comment or a template file extension available

        --engine-list
            Print supported engine list for the `--engine` option

    -h, --help
            Print help information

    -o, --output <OUTPUT FILE>
            Override automatic output file path with the specified file path

    -O, --open
            Open the rendered output file with a default software

    -s, --stdout
            Print rendered result to STDOUT

    -s, --stderr
            Print rendered result to STDERR

    -v, --verbose
            Set the level of verbosity.

            `-v` sets logging level to INFO `-vv` sets logging level to DEBUG

            WARNING: Effects CLI / STDOUT output. Use the `--output` switch if you wish to commit
            the rendered output to file. Use the `--stderr` switch to avoid including the logger
            messages in the final output.

    -V, --version
            Print version information

    -w, --watch
            Constantly render changes in the template with the context file for every 2 seconds
```

</details>

## Supported Template Engines

| Name       | Short  | Version | Guide / Manual / Tutorial                                     |  
| ---------- | ------ | ------- | ------------------------------------------------------------- |
| Tera       | `tera` | v1.16.0 | <https://tera.netlify.app/docs/#templates>                    |
| Handlebars | `hbs`  | v4.3.3  | <https://handlebarsjs.com/guide/>                             |
| Liquid     | `liq`  | v0.26.0 | <https://github.com/Shopify/liquid/wiki/Liquid-for-Designers> |

## Template Examples

<details>
<summary>Tera</summary>

A good alternative choice if you are used to template engines such as `Jinja2`, `Django`, `Liquid` or `Twig`.  
The `Tera` rendering engine is highly advanced, capable and secure rendering engine that follows the OWASP Top 10 guidelines to provide trust and security.

* Guide: <https://tera.netlify.app/docs/#templates>  
* Supported Version: **v1.16.0**

```html
<HTML>
    WIP
</HTML>
```

</details>

<details>
<summary>Handlebars</summary>

The most popular rendering engine that is shared among multiple programming languages. Somewhat more limited than other options.

* Guide: <https://handlebarsjs.com/guide/>  
* Supported Version: **v4.3.3**
  
```html
<HTML>
    WIP
</HTML>
```

</details>

<details>
<summary>Liquid</summary>

A highly advanced rendering engine, coming from the Ruby programming language.

* Guide: <https://github.com/Shopify/liquid/wiki/Liquid-for-Designers>  
* Supported Version: **v0.26.0**

```html
<HTML>
    WIP
</HTML>
```

</details>
