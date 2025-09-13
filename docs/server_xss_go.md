# server-xss-go

Detects potential server-side cross-site scripting (XSS) vulnerabilities in Go
HTTP handlers. The scanner flags two common issues:

1. Using `text/template` for HTML responses instead of `html/template`, which
   does not provide automatic HTML escaping.
2. Writing untrusted input directly to `http.ResponseWriter` without proper
   escaping or templating, such as `w.Write([]byte(r.FormValue("name")))`.

## Recommendation

Use `html/template` for any HTML output and ensure user input is properly
escaped before writing it to the response. Avoid writing request parameters
directly using `fmt.Fprintf`, `io.WriteString`, or `w.Write`.

## Configuration

```toml
[rules.server-xss-go]
enabled = true
severity = "medium"
```
