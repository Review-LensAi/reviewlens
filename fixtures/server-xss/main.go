package main

import (
    "fmt"
    "net/http"
)

func handler(w http.ResponseWriter, r *http.Request) {
    user := r.URL.Query().Get("user")
    fmt.Fprintf(w, "<p>"+user+"</p>")
}
