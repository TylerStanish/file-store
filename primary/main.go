package main

import (
	"database/sql"
	"log"
	"net/http"
	"os"

	"github.com/tylerstanish/file-store/server"

	_ "github.com/lib/pq"
)

func setupDB() *sql.DB {
	db, err := sql.Open("postgres", os.Getenv("POSTGRESQL_URL"))
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

	http.HandleFunc("/tag", server.RequireAuthentication(server.HandleTag))
	http.HandleFunc("/node", server.RequireAuthentication(server.HandleNode))
	http.ListenAndServe(":8080", nil)
}
