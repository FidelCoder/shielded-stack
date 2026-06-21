package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/FidelCoder/shielded-stack/go/internal/probe"
)

func main() {
	endpoints := readEndpoints()
	prober := probe.NewLWDProber(10 * time.Second)

	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})
	mux.HandleFunc("/probe", func(w http.ResponseWriter, r *http.Request) {
		endpoint := r.URL.Query().Get("endpoint")
		if endpoint == "" && len(endpoints) > 0 {
			endpoint = endpoints[0]
		}

		result := prober.Probe(r.Context(), endpoint)
		writeJSON(w, result)
	})
	mux.HandleFunc("/metrics", func(w http.ResponseWriter, r *http.Request) {
		ctx, cancel := context.WithTimeout(r.Context(), 15*time.Second)
		defer cancel()

		writeMetrics(ctx, w, prober, endpoints)
	})

	addr := envOrDefault("SHIELDED_STACK_ADDR", ":9467")
	log.Printf("lwd-exporter listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatal(err)
	}
}

func readEndpoints() []string {
	raw := os.Getenv("SHIELDED_STACK_ENDPOINTS")
	if raw == "" {
		return nil
	}

	items := strings.Split(raw, ",")
	endpoints := make([]string, 0, len(items))
	for _, item := range items {
		item = strings.TrimSpace(item)
		if item != "" {
			endpoints = append(endpoints, item)
		}
	}

	return endpoints
}

func writeJSON(w http.ResponseWriter, value any) {
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(value); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func writeMetrics(ctx context.Context, w http.ResponseWriter, prober probe.LWDProber, endpoints []string) {
	w.Header().Set("Content-Type", "text/plain; version=0.0.4")

	fmt.Fprintln(w, "# HELP shielded_stack_configured_endpoints Number of configured endpoints.")
	fmt.Fprintln(w, "# TYPE shielded_stack_configured_endpoints gauge")
	fmt.Fprintf(w, "shielded_stack_configured_endpoints %d\n", len(endpoints))

	if len(endpoints) == 0 {
		return
	}

	fmt.Fprintln(w, "# HELP shielded_stack_endpoint_reachable Whether the lightwalletd gRPC probe succeeded.")
	fmt.Fprintln(w, "# TYPE shielded_stack_endpoint_reachable gauge")
	fmt.Fprintln(w, "# HELP shielded_stack_endpoint_latency_seconds Endpoint gRPC probe latency in seconds.")
	fmt.Fprintln(w, "# TYPE shielded_stack_endpoint_latency_seconds gauge")
	fmt.Fprintln(w, "# HELP shielded_stack_endpoint_block_height Latest block height reported by lightwalletd.")
	fmt.Fprintln(w, "# TYPE shielded_stack_endpoint_block_height gauge")
	fmt.Fprintln(w, "# HELP shielded_stack_endpoint_estimated_height Estimated block height reported by lightwalletd.")
	fmt.Fprintln(w, "# TYPE shielded_stack_endpoint_estimated_height gauge")
	fmt.Fprintln(w, "# HELP shielded_stack_endpoint_height_lag Estimated height minus reported block height.")
	fmt.Fprintln(w, "# TYPE shielded_stack_endpoint_height_lag gauge")

	for _, endpoint := range endpoints {
		result := prober.Probe(ctx, endpoint)
		reachable := 0
		if result.Reachable {
			reachable = 1
		}

		heightLag := int64(result.EstimatedBlockHeight) - int64(result.LatestBlockHeight)
		if heightLag < 0 {
			heightLag = 0
		}

		fmt.Fprintf(w, "shielded_stack_endpoint_reachable{endpoint=%q} %d\n", endpoint, reachable)
		fmt.Fprintf(w, "shielded_stack_endpoint_latency_seconds{endpoint=%q} %.6f\n", endpoint, result.Latency.Seconds())
		fmt.Fprintf(w, "shielded_stack_endpoint_block_height{endpoint=%q} %d\n", endpoint, result.LatestBlockHeight)
		fmt.Fprintf(w, "shielded_stack_endpoint_estimated_height{endpoint=%q} %d\n", endpoint, result.EstimatedBlockHeight)
		fmt.Fprintf(w, "shielded_stack_endpoint_height_lag{endpoint=%q} %d\n", endpoint, heightLag)
	}
}

func envOrDefault(name, fallback string) string {
	value := strings.TrimSpace(os.Getenv(name))
	if value == "" {
		return fallback
	}

	return value
}
