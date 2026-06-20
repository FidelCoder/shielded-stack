package probe

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestProbeRejectsEmptyEndpoint(t *testing.T) {
	prober := NewHTTPProber(time.Second)
	result := prober.Probe(context.Background(), "")

	if result.Reachable {
		t.Fatal("empty endpoint should not be reachable")
	}
}

func TestProbeReachableServer(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	prober := NewHTTPProber(time.Second)
	result := prober.Probe(context.Background(), server.URL)

	if !result.Reachable {
		t.Fatalf("expected reachable server, got result: %#v", result)
	}
}

