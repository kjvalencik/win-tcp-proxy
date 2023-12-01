# win-tcp-proxy

Async, single threaded TCP proxy for forwarding ports. Runs as a CLI or as a Windows Service.

## Usage

```
Simple TCP Proxy Service

Usage: tcp-proxy.exe <COMMAND>

Commands:
  proxy      Start a TCP proxy
  install    Install a TCP proxy service
  uninstall  Uninstall a TCP proxy service
  service    Should only be executed by a service
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Proxy

Proxy targets are provided as `([bind IP]:)[bind port]:[target hostname]:[target port]`. If a bind IP is not provided, it will use `0.0.0.0`. IPv6 addresses must be wrapped in `[]` (e.g., `[::1]`).

```
tcp-proxy.exe proxy 8080:example.com:80
```

### Service

The proxy may be installed as a Windows Service. Targets use the same format as [`proxy`](#proxy).

```
tcp-proxy.exe install --name my-proxy 8080:example.com:80
```

Installing and uninstalling requires Administrator permissions. The service will be started immediately when installing and stopped when uninstalling.

```
tcp-proxy.exe uninstall --name my-proxy
```

**Note**: The `service` subcommand is only intended to be used by a Windows Service.
