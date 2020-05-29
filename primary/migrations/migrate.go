package main

import (
	"database/sql"
	"io/ioutil"
	"log"
	"os"

	_ "github.com/lib/pq"
)

func main() {
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	db, err := sql.Open("postgres", os.Getenv("POSTGRESQL_URL"))
	if err != nil {
		log.Fatal(err)
	}
	bytes, err := ioutil.ReadFile("migrations/up.sql")
	if err != nil {
		log.Fatal(err)
	}
	stmt, err := db.Prepare(string(bytes))
	if err != nil {
		log.Fatal(err)
	}
	_, err = stmt.Exec()
	if err != nil {
		log.Fatal(err)
	}
	stmt.Close()
}
