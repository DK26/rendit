![render-template](assets/logo.gif)

A template-design aid CLI tool for rendering HTML (and other) template files or piped input, supporting multiple rendering engines.

Can also be used for shell scripting.

## Features

* Rendering (supported) template input either from file or standard input
* Supports piping
* Use either default or specified context JSON file for rendering
* Pretty console output for quick render review
* Output rendered result to file either with a proper default name or a specified one

## Supported Template Engines

| Name       | Short  | Version | Guide                                                         |  
| ---------- | ------ | ------- | ------------------------------------------------------------- |
| Tera       | `tera` | v1.16.0 | <https://tera.netlify.app/docs/#templates>                    |
| Handlebars | `hbs`  | v4.3.2  | <https://handlebarsjs.com/guide/>                             |
| Liquid     | `liq`  | v0.26.0 | <https://github.com/Shopify/liquid/wiki/Liquid-for-Designers> |

## Commandline Usage

Usage Here

### Commandline Parameters

Parameters here

## Template Engines

<details>
<summary>Tera</summary>

A good alternative choice if you are used to template engines such as `Jinja2`, `Django`, `Liquid` or `Twig`.  
The `Tera` rendering engine is highly advanced, capable and secure rendering engine that follows the OWASP Top 10 guidelines to provide trust and security.

* Guide: <https://tera.netlify.app/docs/#templates>  
* Supported Version: **v1.16.0**

```html
<HTML>
</HTML>
```

</details>

<details>
  <summary>Handlebars</summary>

The most popular rendering engine that is shared among multiple programming languages. Somewhat more limited than other options.

* Guide: <https://handlebarsjs.com/guide/>  
* Supported Version: **v4.3.2**
  
```html
<HTML>
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
</HTML>
```

</details>
