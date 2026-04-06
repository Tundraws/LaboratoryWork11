package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestEnvOr(t *testing.T) {
	t.Setenv("LR11_TEST_ENV", "")
	if got := envOr("LR11_TEST_ENV", "fallback"); got != "fallback" {
		t.Fatalf("expected fallback, got %q", got)
	}

	t.Setenv("LR11_TEST_ENV", "  value  ")
	if got := envOr("LR11_TEST_ENV", "fallback"); got != "value" {
		t.Fatalf("expected trimmed value, got %q", got)
	}
}

func TestServiceHandlers(t *testing.T) {
	sharedDir := t.TempDir()
	mux := newMux("go-service", sharedDir)

	t.Run("health", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodGet, "/health", nil)

		mux.ServeHTTP(rr, req)

		if rr.Code != http.StatusOK {
			t.Fatalf("expected status 200, got %d", rr.Code)
		}

		var payload map[string]string
		if err := json.Unmarshal(rr.Body.Bytes(), &payload); err != nil {
			t.Fatalf("decode health payload: %v", err)
		}

		if payload["service"] != "go-service" || payload["status"] != "ok" {
			t.Fatalf("unexpected payload: %#v", payload)
		}
	})

	t.Run("write and shared", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodPost, "/write", nil)

		mux.ServeHTTP(rr, req)

		if rr.Code != http.StatusCreated {
			t.Fatalf("expected status 201, got %d", rr.Code)
		}

		data, err := os.ReadFile(filepath.Join(sharedDir, "go-service.txt"))
		if err != nil {
			t.Fatalf("read shared file: %v", err)
		}
		if !strings.Contains(string(data), "manual write") {
			t.Fatalf("expected manual write marker in %q", string(data))
		}

		rr = httptest.NewRecorder()
		req = httptest.NewRequest(http.MethodGet, "/shared", nil)
		mux.ServeHTTP(rr, req)

		if rr.Code != http.StatusOK {
			t.Fatalf("expected status 200, got %d", rr.Code)
		}
		if !strings.Contains(rr.Body.String(), "\"go-service.txt\"") {
			t.Fatalf("expected shared snapshot to include service file, got %s", rr.Body.String())
		}
	})
}
