package probe

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"
)

type Result struct {
	Endpoint   string        `json:"endpoint"`
	Reachable  bool          `json:"reachable"`
	StatusCode int           `json:"status_code,omitempty"`
	Latency    time.Duration `json:"latency"`
	Message    string        `json:"message"`
}

type HTTPProber struct {
	Client *http.Client
}

func NewHTTPProber(timeout time.Duration) HTTPProber {
	return HTTPProber{
		Client: &http.Client{Timeout: timeout},
	}
}

func (p HTTPProber) Probe(ctx context.Context, endpoint string) Result {
	start := time.Now()
	endpoint = strings.TrimSpace(endpoint)

	if endpoint == "" {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   0,
			Message:   "endpoint URL is empty",
		}
	}

	if !strings.HasPrefix(endpoint, "http://") && !strings.HasPrefix(endpoint, "https://") {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   0,
			Message:   "endpoint URL must start with http:// or https://",
		}
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, endpoint, nil)
	if err != nil {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   time.Since(start),
			Message:   err.Error(),
		}
	}

	client := p.Client
	if client == nil {
		defaultClient := http.Client{Timeout: 10 * time.Second}
		client = &defaultClient
	}

	resp, err := client.Do(req)
	if err != nil {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   time.Since(start),
			Message:   normalizeError(err),
		}
	}
	defer resp.Body.Close()

	return Result{
		Endpoint:   endpoint,
		Reachable:  resp.StatusCode >= 200 && resp.StatusCode < 500,
		StatusCode: resp.StatusCode,
		Latency:    time.Since(start),
		Message:    resp.Status,
	}
}

func normalizeError(err error) string {
	if errors.Is(err, context.Canceled) {
		return "probe canceled"
	}

	if errors.Is(err, context.DeadlineExceeded) {
		return "probe timed out"
	}

	return err.Error()
}

