package probe

import (
	"context"
	"crypto/tls"
	"errors"
	"net/url"
	"strings"
	"time"

	"github.com/FidelCoder/shielded-stack/go/walletrpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

type Result struct {
	Endpoint             string        `json:"endpoint"`
	Reachable            bool          `json:"reachable"`
	LatestBlockHeight    uint64        `json:"latest_block_height,omitempty"`
	EstimatedBlockHeight uint64        `json:"estimated_block_height,omitempty"`
	Latency              time.Duration `json:"latency"`
	Vendor               string        `json:"vendor,omitempty"`
	Version              string        `json:"version,omitempty"`
	ChainName            string        `json:"chain_name,omitempty"`
	Message              string        `json:"message"`
}

type LWDProber struct {
	Timeout time.Duration
}

func NewLWDProber(timeout time.Duration) LWDProber {
	return LWDProber{Timeout: timeout}
}

func (p LWDProber) Probe(ctx context.Context, endpoint string) Result {
	started := time.Now()
	endpoint = strings.TrimSpace(endpoint)

	address, useTLS, err := dialTarget(endpoint)
	if err != nil {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   0,
			Message:   err.Error(),
		}
	}

	timeout := p.Timeout
	if timeout <= 0 {
		timeout = 10 * time.Second
	}

	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	dialOptions := []grpc.DialOption{}
	if useTLS {
		dialOptions = append(dialOptions, grpc.WithTransportCredentials(credentials.NewTLS(&tls.Config{MinVersion: tls.VersionTLS12})))
	} else {
		dialOptions = append(dialOptions, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	conn, err := grpc.NewClient(address, dialOptions...)
	if err != nil {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   time.Since(started),
			Message:   normalizeError(err),
		}
	}
	defer conn.Close()

	client := walletrpc.NewCompactTxStreamerClient(conn)
	info, err := client.GetLightdInfo(ctx, &walletrpc.Empty{})
	if err != nil {
		return Result{
			Endpoint:  endpoint,
			Reachable: false,
			Latency:   time.Since(started),
			Message:   normalizeError(err),
		}
	}

	return Result{
		Endpoint:             endpoint,
		Reachable:            true,
		LatestBlockHeight:    info.BlockHeight,
		EstimatedBlockHeight: info.EstimatedHeight,
		Latency:              time.Since(started),
		Vendor:               info.Vendor,
		Version:              info.Version,
		ChainName:            info.ChainName,
		Message:              "ok",
	}
}

func dialTarget(endpoint string) (string, bool, error) {
	if endpoint == "" {
		return "", false, errors.New("endpoint URL is empty")
	}

	parsed, err := url.Parse(endpoint)
	if err != nil {
		return "", false, err
	}

	switch parsed.Scheme {
	case "http":
		return parsed.Host, false, nil
	case "https":
		return parsed.Host, true, nil
	default:
		return "", false, errors.New("endpoint URL must start with http:// or https://")
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
