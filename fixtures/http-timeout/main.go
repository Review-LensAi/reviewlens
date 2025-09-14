package main

import "net/http"

func fetch(url string) (*http.Response, error) {
    client := &http.Client{
        Transport: nil,
    }
    return client.Get(url)
}
