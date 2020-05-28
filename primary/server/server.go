package server

import (
	"database/sql"
	"encoding/json"
	"log"
	"net/http"

	"github.com/tylerstanish/file-store/server/schemas"
	"github.com/tylerstanish/file-store/server/services"
	"github.com/tylerstanish/file-store/server/utils"
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

func (s *Server) HandleLogin(w http.ResponseWriter, req *http.Request) {
	if req.Method != "GET" {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	reqBody := schemas.LoginRequest{}
	if !utils.ReadBodyInto(req.Body, &reqBody, w) {
		return
	}
	profile := s.AuthService.Login(reqBody)
	bytes, err := json.Marshal(profile)
	if err != nil {
		log.Fatal(err)
	}
	w.Write(bytes)
}

func (s *Server) HandleRegister(w http.ResponseWriter, req *http.Request) {}
func (s *Server) HandleTag(w http.ResponseWriter, req *http.Request)      {}
func (s *Server) HandleNode(w http.ResponseWriter, req *http.Request)     {}
