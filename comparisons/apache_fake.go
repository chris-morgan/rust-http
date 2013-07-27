package main

import (
	"net/http"
	"runtime"
)

var helloWorld = []byte("<html><body><h1>It works!</h1>\n<p>This is the default web page for this server.</p>\n<p>The web server software is running but no content has been added, yet.</p>\n</body></html>\n")

func main() {
	runtime.GOMAXPROCS(runtime.NumCPU())

	http.HandleFunc("/", apacheFakeHandler)
	http.ListenAndServe(":8001", nil)
}

func apacheFakeHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")
	w.Header().Set("Server", "Apache/2.2.22 (Ubuntu)")
	w.Header().Set("Last-Modified", "Thu, 05 May 2011 11:46:42 GMT")
	w.Header().Set("ETag", "\"501b29-b1-4a285ed47404a\"")
	w.Header().Set("Accept-Ranges", "bytes")
	w.Header().Set("Content-Length", "177")
	w.Header().Set("Vary", "Accept-Encoding")
	w.Header().Set("X-Pad", "avoid browser bug")

	w.Write(helloWorld)
}
