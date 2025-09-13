use engine::config::Config;
use engine::scanner::{Scanner, ServerXssGoScanner};

#[test]
fn detects_text_template_usage() {
    let scanner = ServerXssGoScanner;
    let content = r#"
        import "text/template"
        func handler(w http.ResponseWriter, r *http.Request) {
            tmpl := template.New("foo")
            tmpl.Execute(w, r.FormValue("name"))
        }
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("handler.go", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].line_number, 2);
}

#[test]
fn detects_unescaped_input_written() {
    let scanner = ServerXssGoScanner;
    let content = r#"
        func handler(w http.ResponseWriter, r *http.Request) {
            w.Write([]byte(r.FormValue("name")))
        }
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("handler.go", content, &config)
        .expect("scan should work");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].line_number, 3);
}

#[test]
fn allows_html_template() {
    let scanner = ServerXssGoScanner;
    let content = r#"
        import "html/template"
        func handler(w http.ResponseWriter, r *http.Request) {
            tmpl := template.Must(template.New("foo").Parse("<p>{{.}}</p>"))
            tmpl.Execute(w, r.FormValue("name"))
        }
    "#;
    let config = Config::default();
    let issues = scanner
        .scan("handler.go", content, &config)
        .expect("scan should work");
    assert!(issues.is_empty());
}
