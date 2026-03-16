# Remote Development

## Overview

DSCode supports remote development via SSH, WSL, containers, and Kubernetes with full feature parity to local development.

## Architecture

```
┌─────────────────────────┐         ┌──────────────────────────┐
│  Local Machine          │         │  Remote Machine          │
│  ┌───────────────────┐  │         │  ┌────────────────────┐  │
│  │ DSCode UI         │  │         │  │ DSCode Server      │  │
│  │ (full UI)         │  │◄───────►│  │ (headless)         │  │
│  └───────────────────┘  │   nng   │  ├────────────────────┤  │
│                         │   over  │  │ File System        │  │
│                         │   TCP   │  │ LSP Servers        │  │
│                         │         │  │ Extensions         │  │
│                         │         │  │ Terminal           │  │
└─────────────────────────┘         └──────────────────────────┘
```

## Connection Types

### 1. SSH Remote

```rust
pub struct SSHRemote {
    connection: ssh2::Session,
    server_process: RemoteProcess,
}

impl SSHRemote {
    pub async fn connect(host: &str, user: &str) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{}:22", host)).await?;
        let mut session = ssh2::Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        // Authenticate
        session.userauth_agent(user)?;

        // Start remote server
        let channel = session.channel_session()?;
        channel.exec("dscode-server --port 7777")?;

        // Connect via nng
        let socket = nng::Socket::new(Protocol::Req0)?;
        socket.dial(&format!("tcp://{}:7777", host))?;

        Ok(Self { connection: session, server_process })
    }
}
```

### 2. WSL Integration

```rust
pub struct WSLRemote {
    distro: String,
}

impl WSLRemote {
    pub async fn connect(distro: &str) -> Result<Self> {
        // Start server in WSL
        Command::new("wsl")
            .args(&["-d", distro, "dscode-server", "--port", "7777"])
            .spawn()?;

        // Connect via localhost
        let socket = nng::Socket::new(Protocol::Req0)?;
        socket.dial("tcp://localhost:7777")?;

        Ok(Self { distro: distro.to_string() })
    }
}
```

### 3. Container/Docker

```rust
pub struct ContainerRemote {
    container_id: String,
}

impl ContainerRemote {
    pub async fn connect(container: &str) -> Result<Self> {
        // Copy server binary to container
        Command::new("docker")
            .args(&["cp", "dscode-server", &format!("{}:/tmp/", container)])
            .output()?;

        // Start server in container
        Command::new("docker")
            .args(&["exec", "-d", container, "/tmp/dscode-server", "--port", "7777"])
            .spawn()?;

        // Get container IP
        let ip = get_container_ip(container)?;

        // Connect
        let socket = nng::Socket::new(Protocol::Req0)?;
        socket.dial(&format!("tcp://{}:7777", ip))?;

        Ok(Self { container_id: container.to_string() })
    }
}
```

## File System Forwarding

```rust
#[derive(Archive, Serialize, Deserialize)]
pub enum VFSMessage {
    ReadFile { path: String },
    WriteFile { path: String, content: Vec<u8> },
    ListDir { path: String },
    Watch { path: String },
}

pub struct RemoteVFS {
    socket: nng::Socket,
}

impl RemoteVFS {
    pub async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let msg = VFSMessage::ReadFile { path: path.to_string_lossy().to_string() };
        let response = self.socket.send(rkyv::to_bytes(&msg)?).await?;
        Ok(rkyv::from_bytes(&response)?)
    }
}
```

## Port Forwarding

```rust
pub struct PortForward {
    local_port: u16,
    remote_port: u16,
    tunnel: TcpListener,
}

impl PortForward {
    pub async fn forward(local: u16, remote: u16, ssh: &SSHRemote) -> Result<Self> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", local)).await?;

        tokio::spawn(async move {
            loop {
                let (local_stream, _) = listener.accept().await?;
                let remote_stream = ssh.connection.channel_direct_tcpip(
                    "localhost", remote, None
                )?;

                // Bidirectional copy
                tokio::io::copy_bidirectional(&mut local_stream, &mut remote_stream).await?;
            }
        });

        Ok(Self { local_port: local, remote_port: remote, tunnel: listener })
    }
}
```

## Features

- ✅ File editing (transparent local caching)
- ✅ Terminal forwarding (full PTY support)
- ✅ Extension execution (remote)
- ✅ LSP servers (run remotely)
- ✅ Port forwarding (expose remote ports locally)
- ✅ Git operations (remote)
- ✅ Debugging (DAP forwarded)
- ✅ Settings sync
