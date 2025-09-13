package main

import "net/http"

func fetch(url string) (*http.Response, error) {
    client := &http.Client{}
    return client.Get(url)
}
