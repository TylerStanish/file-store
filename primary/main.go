package main

import (
	"encoding/json"
	"io"
	"log"
	"net/http"
)

type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

// ReadBodyInto Tries to read the request body into the target,
// but will write an HTTP 400 on failure to do so and will return false.
// Otherwise it will populate reqBodyTarget and return true
func ReadBodyInto(bodyReader io.Reader, reqBodyTarget interface{}, w http.ResponseWriter) bool {
	decoder := json.NewDecoder(bodyReader)
	if err := decoder.Decode(reqBodyTarget); err != nil {
		log.Println(err)
		w.WriteHeader(400)
		return false
	}
	return true
}

func AcceptableMethods(req *http.Request, w http.ResponseWriter, methods ...string) bool {
	for _, method := range methods {
		if method == req.Method {
			return true
		}
	}
	w.WriteHeader(http.StatusMethodNotAllowed)
	return false
}

func handleLogin(w http.ResponseWriter, req *http.Request) {
	if req.Method != "GET" {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	reqBody := LoginRequest{}
	if !ReadBodyInto(req.Body, &reqBody, w) {
		return
	}
	bytes, err := json.Marshal(reqBody)
	if err != nil {
		log.Fatal(err)
	}
	w.Write(bytes)
}

func handleRegister(w http.ResponseWriter, req *http.Request) {}
func handleTag(w http.ResponseWriter, req *http.Request)      {}
func handleNode(w http.ResponseWriter, req *http.Request)     {}

func main() {
	http.HandleFunc("/auth/login", handleLogin)
	http.HandleFunc("/auth/register", handleRegister)
	http.HandleFunc("/tag", handleTag)
	http.HandleFunc("/node", handleNode)
	http.ListenAndServe(":3000", nil)
}
