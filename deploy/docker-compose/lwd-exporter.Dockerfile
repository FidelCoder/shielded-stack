FROM golang:1.24 AS build

WORKDIR /src
COPY go.mod go.sum ./
COPY go ./go
RUN go build -trimpath -ldflags "-s -w" -o /out/lwd-exporter ./go/cmd/lwd-exporter

FROM gcr.io/distroless/base-debian12

COPY --from=build /out/lwd-exporter /usr/local/bin/lwd-exporter
EXPOSE 9467
ENTRYPOINT ["/usr/local/bin/lwd-exporter"]
