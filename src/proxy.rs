use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use clap::Parser;
use color_eyre::eyre::{ContextCompat, Error, WrapErr};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinSet,
};

#[derive(Clone, Parser, Debug)]
pub(crate) struct Proxy {
    #[arg(required = true, num_args = 1..)]
    /// Proxy targets (e.g., 8080:192.168.0.1:80)
    targets: Vec<Target>,
}

impl Proxy {
    pub(crate) async fn run(self) -> Result<(), Error> {
        let mut set = JoinSet::new();

        for target in self.targets {
            set.spawn(target.proxy());
        }

        set.join_next().await.unwrap_or(Ok(Ok(())))??;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Target {
    dest: (String, u16),
    source: SocketAddr,
}

impl Target {
    async fn proxy(self) -> Result<(), Error> {
        let listener = TcpListener::bind(self.source).await?;
        let dest = Arc::new(self.dest);

        loop {
            let (stream, addr) = listener.accept().await?;
            let dest = dest.clone();

            log::trace!("Proxying {} to {:?}", addr, dest.as_ref());

            tokio::spawn(async move {
                if let Err(err) = proxy((&dest.0, dest.1), stream).await {
                    log::error!("Error proxying: {err}");
                } else {
                    log::trace!("Proxy for {addr} closed");
                }
            });
        }
    }
}

async fn proxy(dest: (&str, u16), mut stream: TcpStream) -> Result<(), Error> {
    let mut dest = TcpStream::connect(dest).await?;

    tokio::io::copy_bidirectional(&mut stream, &mut dest).await?;

    Ok(())
}

impl FromStr for Target {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, dest_port) = s.rsplit_once(':').context("Missing destination port")?;
        let dest_port = dest_port
            .parse::<u16>()
            .context("Invalid destination port")?;

        // Parse IPv6
        let (s, dest) = if s.ends_with(']') {
            let (s, dest_addr) = s.rsplit_once('[').context("Missing destination address")?;
            let dest_addr = dest_addr.trim_end_matches(']');
            let s = s.trim_end_matches(':');

            (s, dest_addr)

            // IPv4 or hostname
        } else {
            s.rsplit_once(':').context("Missing destination address")?
        };

        let (source_addr, source_port) = s.rsplit_once(':').unwrap_or(("0.0.0.0", s));
        let source_addr = source_addr.trim_start_matches('[').trim_end_matches(']');
        let source_addr = source_addr
            .parse::<IpAddr>()
            .context("Invalid source address")?;

        let source_port = source_port.parse::<u16>().context("Invalid source port")?;
        let source = SocketAddr::from((source_addr, source_port));

        Ok(Self {
            dest: (dest.to_owned(), dest_port),
            source,
        })
    }
}

#[test]
fn target_parse() {
    assert_eq!(
        "8080:192.168.0.100:80".parse::<Target>().unwrap(),
        Target {
            source: "0.0.0.0:8080".parse().unwrap(),
            dest: (String::from("192.168.0.100"), 80),
        }
    );

    assert_eq!(
        "127.0.0.1:8080:192.168.0.100:80".parse::<Target>().unwrap(),
        Target {
            source: "127.0.0.1:8080".parse().unwrap(),
            dest: (String::from("192.168.0.100"), 80),
        }
    );

    assert_eq!(
        "8080:[::1]:80".parse::<Target>().unwrap(),
        Target {
            source: "0.0.0.0:8080".parse().unwrap(),
            dest: (String::from("::1"), 80),
        }
    );

    assert_eq!(
        "[::1]:8080:[::1]:80".parse::<Target>().unwrap(),
        Target {
            source: "[::1]:8080".parse().unwrap(),
            dest: (String::from("::1"), 80),
        }
    );

    assert_eq!(
        "8080:localhost:80".parse::<Target>().unwrap(),
        Target {
            source: "0.0.0.0:8080".parse().unwrap(),
            dest: (String::from("localhost"), 80),
        }
    );
}
