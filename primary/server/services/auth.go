package services

import (
	"database/sql"
	"fmt"
	"log"
	"time"

	"github.com/google/uuid"
)

type AuthService struct {
	DBClient *sql.DB
}

type Token struct {
	Token    string
	IssuedAt time.Time
}

func tokenCols() string {
	return "token, issued_at"
}

func tokenFields(t *Token) []interface{} {
	return []interface{}{&t.Token, &t.IssuedAt}
}

func (m *AuthService) CreateApiKey() *Token {
	token := Token{}
	tokStr := uuid.New().Value
	err := m.DBClient.QueryRow(
		fmt.Sprintf("insert into token (token) values ($1) returning %s", tokenCols()),
		tokStr,
	).Scan(tokenFields(&token)...)
	if err != nil {
		log.Fatal(err)
	}
	return &token
}
