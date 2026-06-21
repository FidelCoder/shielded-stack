package probe

import "testing"

func TestDialTargetRejectsEmptyEndpoint(t *testing.T) {
	_, _, err := dialTarget("")
	if err == nil {
		t.Fatal("empty endpoint should fail")
	}
}

func TestDialTargetRejectsUnsupportedScheme(t *testing.T) {
	_, _, err := dialTarget("tcp://example.invalid:9067")
	if err == nil {
		t.Fatal("unsupported scheme should fail")
	}
}

func TestDialTargetParsesHTTPSEndpoint(t *testing.T) {
	address, useTLS, err := dialTarget("https://example.invalid:9067")
	if err != nil {
		t.Fatalf("expected endpoint to parse: %v", err)
	}

	if address != "example.invalid:9067" {
		t.Fatalf("unexpected address: %s", address)
	}

	if !useTLS {
		t.Fatal("https endpoint should use TLS")
	}
}

func TestDialTargetParsesHTTPEndpoint(t *testing.T) {
	address, useTLS, err := dialTarget("http://127.0.0.1:9067")
	if err != nil {
		t.Fatalf("expected endpoint to parse: %v", err)
	}

	if address != "127.0.0.1:9067" {
		t.Fatalf("unexpected address: %s", address)
	}

	if useTLS {
		t.Fatal("http endpoint should not use TLS")
	}
}
