![render-template](assets/logo.gif)

Renders templates and their context offline automatically, supporting multiple engines.

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
            The template file path requiring a `default.ctx.json` context file or template specific
            context file containing the template name and ending with the `.ctx.json` extension:
            
            e.g. For the Template file `my_template.html` the context file would be
            `my_template.ctx.json` When both are located under the same directory.
            
            If `my_template.ctx.json` is missing, the tool will attempt to load `default.ctx.json`
            under the same directory.
            
            Output: - Providing `<TEMPLATE FILE>` file will automatically produce a rendered output
            file with a proper name and extension: `<TEMPLATE NAME>.rendered.<extension>`. - NOT
            providing `<TEMPLATE FILE>`, will trigger STDIN mode and will attempt to read the
            template data from STDIN, WITHOUT producing an output file.

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
            Sets the level of verbosity.
            
            `-v` sets logging level to INFO `-vv` sets logging level to DEBUG
            
            WARNING: Effects CLI / STDOUT output. Use the `--output` switch if you wish to commit
            the rendered output to file. Use the `--stderr` switch to avoid including the logger
            messages in the final output.

    -V, --version
            Print version information

    -w, --watch
            Constantly render changes of both the template and the context files for every 2 seconds

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
