package main

import (
	"database/sql"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/tylerstanish/file-store/server"

	_ "github.com/lib/pq"
)

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
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	db := setupDB()
	defer db.Close()

	server := server.NewServer(db)

	http.HandleFunc("/auth/login", server.HandleLogin)
	http.HandleFunc("/auth/register", server.HandleRegister)
	http.HandleFunc("/tag", server.HandleTag)
	http.HandleFunc("/node", server.HandleNode)
	http.ListenAndServe(":3000", nil)
}
