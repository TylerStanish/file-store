package server

import (
	"database/sql"
	"net/http"

	"github.com/tylerstanish/file-store/server/services"
)

type Server struct {
	DBClient    *sql.DB
	AuthService *services.AuthService
}

func NewServer(db *sql.DB) *Server {
	return &Server{
		DBClient:    db,
		AuthService: &services.AuthService{DBClient: db},
	}
}

func (s *Server) HandleTag(w http.ResponseWriter, req *http.Request)  {}
func (s *Server) HandleNode(w http.ResponseWriter, req *http.Request) {}
