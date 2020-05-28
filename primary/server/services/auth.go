package services

import (
	"database/sql"
	"fmt"
	"log"

	"github.com/tylerstanish/file-store/server/schemas"
)

type AuthService struct {
	DBClient *sql.DB
}

type Profile struct {
	Username string
	Password string
}

func profileCols() string {
	return "username, password"
}

func (m *AuthService) Login(req schemas.LoginRequest) *Profile {
	profile := Profile{}
	err := m.DBClient.QueryRow(fmt.Sprintf("select %s from profile", profileCols())).Scan(&profile.Username, &profile.Password)
	if err != nil {
		log.Fatal(err)
	}
	return &profile
}
