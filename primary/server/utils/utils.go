package utils

import (
	"encoding/json"
	"io"
	"log"
	"net/http"
)

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
