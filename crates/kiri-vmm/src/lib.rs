//! Linux host runtime agent transport: Caskel runs containers directly on
//! the host kernel. The agent is a managed root process connected to the
//! daemon via private Unix sockets; no guest VM or Apple framework is used.

#[cfg(not(target_os = "linux"))]
compile_error!("Caskel's agent transport supports Linux only");

mod stop;
#[cfg(target_os = "linux")]
mod process;
#[cfg(target_os = "linux")]
pub use process::*;
