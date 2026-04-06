FROM golang:1.24-alpine AS builder

WORKDIR /src

COPY go.mod ./
RUN go mod download

COPY cmd ./cmd

RUN CGO_ENABLED=0 GOOS=linux go build -o /out/go-service ./cmd/server

FROM alpine:3.22

RUN apk add --no-cache ca-certificates wget

WORKDIR /app

COPY --from=builder /out/go-service /usr/local/bin/go-service

EXPOSE 8080

CMD ["go-service"]
