package services

import (
	"database/sql"
	"fmt"
	"log"
	"time"

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

type Token struct {
	Token     string
	IssuedAt  time.Time
	ProfileId int
}

func tokenCols() string {
	return "token, issued_at, profile_id"
}

func tokenFields(t *Token) []interface{} {
	return []interface{}{&t.Token, &t.IssuedAt, &t.ProfileId}
}

func (m *AuthService) Register(req schemas.RegisterRequest) *Profile {
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

func (m *AuthService) Login(req schemas.RegisterRequest) *Token {
	token := Token{}
	err := m.DBClient.QueryRow(
		fmt.Sprintf("select * from ")
	)
}
