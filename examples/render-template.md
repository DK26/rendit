
# usage 1

// Uses `default.ctx.json` file for context  
`type my_template.html | render-template > my_template.rendered.html`

`type my_template.html | render-template -c context.json > my_template.rendered.html`

`type ./my_template.html | render-template --output ./my_template.rendered.html | grep *`

`type my_template.html | render-template --engine tera > my_template.rendered.html`

*Missing input `<TEMPLATE FILE>` ? -check STDIN + print to STDOUT by default instead of file + If missing `-c` or `--context-file`, try default `default.ctx.json` file. If missing Exit with explanation  

---

# usage 2

* Every default value can be overridden with a switch: `--context-file` and/or `--output`

* If `--context-file` is activated, use it for context,  
	else, if my_template.ctx.html` exists, use it for context,  
	else, if `default.ctx.html` exists, use it for context,  
	else, error: No context was provided  

`render-template ./my_template.html --output ./other_place.rendered.html --stdout > ./my_template.rendered.html`

`render-template ./my_template.html --context-file ./my_template.json --stdout > ./my_template.rendered.html`

`render-template ./my_template.html --context-file ./my_template.json --pretty-stdout`

`render-template ./my_template.html --pretty-stdout`

---

# usage 3

`render-template ./my_template.html --output ./my_template.rendered.html`

`render-template ./my_template.html --context-file ./my_template.json --output ./my_template.rendered.html --stdout | send-tcp 192.168.0.1:445`

# usage 4

`render-template ./my_template.html --stdout --output ./my_template.rendered.html`

`type my_template.html | render-template --output ./my_template.rendered.html ---pretty-stdout`

`type my_template.html | render-template --output ./my_template.rendered.html --stdout | grep *`

// `--stdout` switch is auto-activated when no `<TEMPLATE FILE>` is provided
`type my_template.html | render-template --output ./my_template.rendered.html | grep *`

`type my_template.html | render-template --stdout | grep *`

`type my_template.html | render-template | grep *`

# usage 5  

// Warn that when using `--verbose` we should not rely on stdout as verbose produces extra values.
`render-template ./my_template.html -v -o ./some_file.rendered.html`

# usage 6

// New subcommand
// Create a new template skeleton (default: HTML) with magic comment (default: tera) and context file
`render-template new my_template.html --engine tera --newdir`

---

`template render my_template.html`
`template new my_template.html`
