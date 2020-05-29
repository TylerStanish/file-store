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

func profileFields(p *Profile) []interface{} {
	return []interface{}{&p.Username, &p.Password}
}

func (m *AuthService) Login(req schemas.LoginRequest) *Profile {
	profile := Profile{}
	err := m.DBClient.QueryRow(
		fmt.Sprintf("insert into profile (username, password) values ($1, $2) returning %s", profileCols()),
		req.Username,
		req.Password,
	).Scan(profileFields(&profile)...)
	if err != nil {
		log.Fatal(err)
	}
	return &profile
}
