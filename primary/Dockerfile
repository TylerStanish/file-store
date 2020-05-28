FROM golang:buster
WORKDIR /go/src/github.com/app

COPY go.mod .
COPY go.sum .

RUN go mod download

RUN go get github.com/githubnemo/CompileDaemon

COPY . .