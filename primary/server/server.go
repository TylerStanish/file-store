package server

import (
	"database/sql"
	"log"
	"net/http"

	"github.com/tylerstanish/file-store/server/services"
)

type Server struct {
	DBClient    *sql.DB
	AuthService *services.AuthService
}

type Route = func(http.ResponseWriter, *http.Request)

func (s *Server) RequireAuthentication(next func(w http.ResponseWriter, req *http.Request)) Route {
	return func(r http.ResponseWriter, req *http.Request) {
		authHeader := req.Header.Get("Authorization")
		if authHeader == "" {
			r.WriteHeader(http.StatusBadRequest)
			return
		}
		rows, err := s.DBClient.Query("select true from token where token = $1", authHeader)
		defer rows.Close()
		if err != nil {
			log.Fatal(err)
		}
		if !rows.Next() {
			r.WriteHeader(http.StatusUnauthorized)
		}
		next(r, req)
	}
}

func NewServer(db *sql.DB) *Server {
	return &Server{
		DBClient:    db,
		AuthService: &services.AuthService{DBClient: db},
	}
}

func (s *Server) HandleTag(w http.ResponseWriter, req *http.Request) {
	if req.Method == "GET" {
		w.Write([]byte("hi"))
		return
	}
	if req.Method != "POST" {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
}
func (s *Server) HandleNode(w http.ResponseWriter, req *http.Request) {}
