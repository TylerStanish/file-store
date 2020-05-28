package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
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

func setupDB() *sql.DB {
	db, err := sql.Open("postgres", fmt.Sprintf(
		"user=%s host=%s dbname=%s password=%s port=%s sslmode=disable", // TODO lol please change sslmode
		os.Getenv("DBUSER"),
		os.Getenv("DBHOST"),
		os.Getenv("DBNAME"),
		os.Getenv("DBPASS"),
		os.Getenv("DBPORT"),
	))
	if err != nil {
		log.Fatal(err)
	}
	if err := db.Ping(); err != nil {
		log.Fatal(err)
	}
	return db
}

func main() {
	db := setupDB()
	defer db.Close()

	http.HandleFunc("/auth/login", handleLogin)
	http.HandleFunc("/auth/register", handleRegister)
	http.HandleFunc("/tag", handleTag)
	http.HandleFunc("/node", handleNode)
	http.ListenAndServe(":3000", nil)
}
