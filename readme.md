note: this project is a fork of [caheredia/poolboy](https://github.com/caheredia/poolboy) which publishes an html table containing p2pool metrics. it has been forked and adapted to serve metrics in a format digestble by prometheus and renamed to reflect the purpose of the fork.

# p2pool-exporter
![ci tests](https://github.com/grenade/p2pool-exporter/actions/workflows/rust.yml/badge.svg)

a prometheus exporter of p2pool metrics from your local [monero p2pool](https://github.com/SChernykh/p2pool), a decentralized pool for [monero](https://github.com/monero-project/monero) mining.

## installation
```bash
cargo install --git https://github.com/grenade/p2pool-exporter
```

## run the server
```bash
${HOME}/.cargo/bin/p2pool-exporter \
    --data-dir /var/lib/p2pool \
    --listen-ip 127.0.0.1 \
    --listen-port 18090 \
    --metrics-path /metrics
```

⚠️ it is important that the value of `--data-dir` should match the path used by p2pool's `--data-api` parameter.

## connecting to the server
point your browser at:
- [http://127.0.0.1:18090/json](http://127.0.0.1:18090/json)
- [http://127.0.0.1:18090/metrics](http://127.0.0.1:18090/metrics)

## options
```console
❯ p2pool-exporter --help
a prometheus exporter of p2pool metrics

Usage: p2pool-exporter [OPTIONS]

Options:
  -h, --help     Print help
  -V, --version  Print version

p2pool:
  -d, --data-directory <data directory>  the p2pool data directory [default: /var/lib/p2pool]

http server:
  -i, --listen-ip <ip address>       the ip address to listen on [default: 127.0.0.1]
  -p, --listen-port <port>           the port to listen on [default: 18090]
  -m, --metrics-path <metrics path>  the path portion of the url to prometheus metrics [default: /metrics]

```
