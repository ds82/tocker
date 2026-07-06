use anyhow::{Context, Result};
use bollard::{Docker, API_DEFAULT_VERSION};
use std::env;

pub struct SshTunnel {
    _child: tokio::process::Child,
    pub socket_path: String,
}

impl SshTunnel {
    pub async fn establish(target: &str) -> Result<Self> {
        let host = target.strip_prefix("ssh://").unwrap_or(target);
        let socket_path = format!("/tmp/tocker-{}.sock", std::process::id());
        let _ = std::fs::remove_file(&socket_path);

        let child = tokio::process::Command::new("ssh")
            .args([
                "-N",
                "-L",
                &format!("{socket_path}:/var/run/docker.sock"),
                host,
            ])
            .kill_on_drop(true)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("failed to spawn ssh — is OpenSSH installed?")?;

        // Poll until the forwarded socket appears (max 10s)
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(10);
        loop {
            if std::path::Path::new(&socket_path).exists() {
                break;
            }
            if tokio::time::Instant::now() > deadline {
                anyhow::bail!(
                    "SSH tunnel to {host} timed out — check host reachability \
                     and Docker socket permissions"
                );
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(Self {
            _child: child,
            socket_path,
        })
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

pub async fn connect() -> Result<(Docker, Option<SshTunnel>)> {
    match env::var("DOCKER_HOST").ok().as_deref() {
        None | Some("") => {
            let docker = Docker::connect_with_local_defaults()
                .context("failed to connect to local Docker socket")?;
            Ok((docker, None))
        }
        Some(host) if host.starts_with("unix://") => {
            let path = host.strip_prefix("unix://").unwrap();
            let docker = Docker::connect_with_socket(path, 120, API_DEFAULT_VERSION)
                .context("failed to connect to Unix socket")?;
            Ok((docker, None))
        }
        Some(host) if host.starts_with("tcp://") => {
            let docker = Docker::connect_with_http(host, 120, API_DEFAULT_VERSION)
                .context("failed to connect via TCP")?;
            Ok((docker, None))
        }
        Some(host) => {
            // Bare hostname, user@host, or ssh://... → SSH tunnel
            let tunnel = SshTunnel::establish(host).await?;
            let docker = Docker::connect_with_socket(&tunnel.socket_path, 120, API_DEFAULT_VERSION)
                .context("failed to connect to tunnelled Docker socket")?;
            Ok((docker, Some(tunnel)))
        }
    }
}
