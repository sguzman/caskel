//! A running Caskel Linux host agent and its control connection.

// The control API retains historic guest/VM terminology while the Linux
// agent transport is separated from the old Apple VM implementation.
#![allow(dead_code)]

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use kiri_proto::{HEADER, Hello, LayerBegin, Msg, Stream};
use kiri_vmm::{Config, Event, Vm};

/// How long each boot stage took, measured from the host.
#[derive(Debug, Clone, Copy)]
pub struct BootTimings {
    /// Host-side time until the Linux agent process is launched.
    pub vm_started: Duration,
    /// Host-side time until the guest agent's Hello arrived.
    pub agent_ready: Duration,
    /// Guest-side: kernel start to PID 1.
    pub kernel: Duration,
    /// Guest-side: PID 1 start to agent ready.
    pub init: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct Latency {
    pub p50: Duration,
    pub p99: Duration,
}

#[derive(Debug, Default)]
pub struct BootOptions {
    /// Send the guest console to stdout.
    pub console: bool,
    /// Keep a bounded, redacted guest console in this file.
    pub console_log: Option<PathBuf>,
    /// Attach the persistent data disk (layers and containers). Without it
    /// the guest keeps them in memory, which is fine for a quick boot test.
    pub data_disk: bool,
    /// Retained for callers; Linux enforces memory limits through cgroups.
    pub memory_mib: Option<u64>,
    /// Retained for callers; Linux uses the host kernel.
    pub cpus: Option<usize>,
}

fn console_output(opts: &BootOptions) -> io::Result<Option<OwnedFd>> {
    if let Some(path) = &opts.console_log {
        crate::diag::console::capture(path).map(Some)
    } else if opts.console {
        io::stdout().as_fd().try_clone_to_owned().map(Some)
    } else {
        Ok(None)
    }
}

/// One container run, as the host asks for it.
#[derive(Debug)]
pub struct RunRequest<'a> {
    pub id: &'a str,
    /// Layer digests from the top layer down to the base.
    pub layers: &'a [String],
    pub argv: &'a [String],
    pub env: &'a [String],
    pub cwd: &'a str,
    pub user: &'a str,
    pub hostname: &'a str,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RunResult {
    pub code: i32,
    /// Host-side: request sent until the exit arrived.
    pub total: Duration,
    /// Guest-side: request received until the process was cloned.
    pub guest_setup: Duration,
    /// Guest-side: clone until the exit was reaped.
    pub guest_run: Duration,
}

#[derive(Debug)]
pub struct Guest {
    vm: Vm,
    conn: File,
    /// Reused for every frame, so steady-state traffic does not allocate.
    frame: Vec<u8>,
    out: Vec<u8>,
    rtts: Vec<Duration>,
    seq: u64,
    pub hello: Hello,
    pub timings: BootTimings,
}

impl Guest {
    pub fn boot(opts: &BootOptions) -> io::Result<Guest> {
        if !Vm::is_supported() {
            return Err(io::Error::other(
                "Caskel's Linux host agent requires root privileges: start caskel.service or run caskel daemon as root",
            ));
        }
        let cfg = Config {
            agent: agent_binary(),
            console: console_output(opts)?,
            agent_port: kiri_proto::HOST_PORT,
        };
        Self::start(cfg)
    }

    fn start(cfg: Config) -> io::Result<Guest> {
        let t0 = Instant::now();
        let vm = Vm::start(cfg)?;
        let vm_started = t0.elapsed();
        let conn = match vm.next_event(Duration::from_secs(10)) {
            Some(Event::GuestConnected(fd)) => File::from(fd),
            Some(Event::Stopped(why)) => {
                return Err(io::Error::other(format!("guest stopped before connecting: {why:?}")));
            }
            None => return Err(io::Error::new(io::ErrorKind::TimedOut, "guest did not connect within 10 s")),
        };
        let mut guest = Guest {
            vm,
            conn,
            frame: Vec::with_capacity(256),
            out: Vec::with_capacity(64),
            rtts: Vec::new(),
            seq: 0,
            hello: Hello { protocol: 0, init_start_ns: 0, ready_ns: 0, mem_total_kib: 0, mem_available_kib: 0 },
            timings: BootTimings {
                vm_started,
                agent_ready: Duration::ZERO,
                kernel: Duration::ZERO,
                init: Duration::ZERO,
            },
        };
        guest.hello = match guest.read()? {
            Msg::Hello(h) => h,
            other => return Err(io::Error::other(format!("expected Hello, got {other:?}"))),
        };
        // The agent comes from the same install, so another version means the
        // files changed under a running Kiri: stop rather than misread it.
        if guest.hello.protocol != kiri_proto::PROTOCOL {
            return Err(io::Error::other(format!(
                "the agent speaks protocol {}, this Caskel daemon speaks {}: restart Caskel using one version",
                guest.hello.protocol,
                kiri_proto::PROTOCOL
            )));
        }
        guest.timings.agent_ready = t0.elapsed();
        // The guest has no real-time clock source of its own.
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        guest.send(Msg::SetTime(now.as_nanos() as u64))?;
        guest.timings.kernel = Duration::from_nanos(guest.hello.init_start_ns);
        guest.timings.init = Duration::from_nanos(guest.hello.ready_ns - guest.hello.init_start_ns);
        Ok(guest)
    }

    /// Measures `n` sequential round trips over the agent connection.
    pub fn ping(&mut self, n: usize) -> io::Result<Latency> {
        self.rtts.clear();
        self.rtts.reserve(n);
        for _ in 0..n {
            self.seq += 1;
            let seq = self.seq;
            let s = Instant::now();
            self.send(Msg::Ping(seq))?;
            match self.read()? {
                Msg::Pong(v) if v == seq => {}
                other => return Err(io::Error::other(format!("expected Pong({seq}), got {other:?}"))),
            }
            self.rtts.push(s.elapsed());
        }
        self.rtts.sort_unstable();
        let pct = |p: usize| self.rtts.get((self.rtts.len() * p / 100).min(self.rtts.len().saturating_sub(1)));
        Ok(Latency {
            p50: pct(50).copied().unwrap_or_default(),
            p99: pct(99).copied().unwrap_or_default(),
        })
    }

    pub fn has_layer(&mut self, digest: &str) -> io::Result<bool> {
        self.send(Msg::LayerQuery(digest))?;
        match self.read()? {
            Msg::LayerStatus(have) => Ok(have),
            other => Err(unexpected("LayerStatus", &other)),
        }
    }

    /// Streams a compressed layer blob to the guest, which unpacks it.
    /// Returns the unpacked size.
    pub fn send_layer(&mut self, digest: &str, media_type: &str, blob: &std::path::Path) -> io::Result<u64> {
        let mut file = File::open(blob)?;
        let size = file.metadata()?.len();
        self.send(Msg::LayerBegin(LayerBegin { digest, media_type, size }))?;
        let mut buf = vec![0u8; 256 * 1024];
        let mut left = size;
        while left > 0 {
            let want = left.min(buf.len() as u64) as usize;
            let n = file.read(&mut buf[..want])?;
            if n == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "layer blob shrank while sending"));
            }
            self.conn.write_all(&buf[..n])?;
            left -= n as u64;
        }
        match self.read()? {
            Msg::LayerStored(s) => Ok(s.unpacked_bytes),
            Msg::Error(e) => Err(io::Error::other(e.to_owned())),
            other => Err(unexpected("LayerStored", &other)),
        }
    }

    /// Runs one container to completion, passing its output to `on_output`.
    pub fn run(&mut self, req: &RunRequest<'_>, mut on_output: impl FnMut(Stream, &[u8])) -> io::Result<RunResult> {
        self.out.clear();
        let none: &[String] = &[];
        let net = kiri_proto::Net { bridge: "", ip: [0; 4], prefix: 0, gateway: [0; 4], extra_hosts: none, dns_names: none, opts: none };
        kiri_proto::encode_run(
            &mut self.out, req.id, req.layers, req.argv, req.env, req.cwd, req.user, req.hostname, req.flags, net, none, &kiri_proto::Limits::default(), 0, none, &[],
        );
        let t = Instant::now();
        self.conn.write_all(&self.out)?;
        loop {
            match self.read()? {
                Msg::Started { .. } => {}
                Msg::Output { stream, data, .. } => on_output(stream, data),
                Msg::Exited(e) => {
                    return Ok(RunResult {
                        code: e.code,
                        total: t.elapsed(),
                        guest_setup: Duration::from_nanos(e.setup_ns),
                        guest_run: Duration::from_nanos(e.run_ns),
                    });
                }
                Msg::Error(e) => return Err(io::Error::other(e.to_owned())),
                other => return Err(unexpected("container output", &other)),
            }
        }
    }

    /// Asks the guest to power off and waits for the VM to stop. Returns how
    /// long that took.
    pub fn shutdown(mut self) -> io::Result<Duration> {
        let t = Instant::now();
        self.send(Msg::Shutdown)?;
        loop {
            match self.vm.next_event(Duration::from_secs(10)) {
                Some(Event::Stopped(None)) => return Ok(t.elapsed()),
                Some(Event::Stopped(Some(why))) => return Err(io::Error::other(why)),
                Some(Event::GuestConnected(_)) => continue,
                None => {
                    self.vm.force_stop();
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "guest did not power off; forced"));
                }
            }
        }
    }

    /// Hands over the VM and the agent connection, for callers that drive the
    /// connection from several threads (the daemon).
    pub fn into_parts(self) -> (Vm, File) {
        (self.vm, self.conn)
    }

    fn send(&mut self, msg: Msg<'_>) -> io::Result<()> {
        self.out.clear();
        msg.encode(&mut self.out);
        self.conn.write_all(&self.out)
    }

    fn read(&mut self) -> io::Result<Msg<'_>> {
        self.frame.resize(4, 0);
        self.conn.read_exact(&mut self.frame[..4])?;
        let len = kiri_proto::frame_len(self.frame[..4].try_into().unwrap())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;
        self.frame.resize(len.max(HEADER), 0);
        self.conn.read_exact(&mut self.frame[4..])?;
        kiri_proto::decode(&self.frame).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))
    }
}

fn unexpected(wanted: &str, got: &Msg<'_>) -> io::Error {
    io::Error::other(format!("expected {wanted}, got {got:?}"))
}

/// The agent on Linux: `KIRI_AGENT`, else `kiri-init` beside `caskel`.
fn agent_binary() -> PathBuf {
    if let Some(p) = std::env::var_os("KIRI_AGENT") {
        return PathBuf::from(p);
    }
    std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("kiri-init"))).unwrap_or_else(|| PathBuf::from("kiri-init"))
}
