//! Manual-IP networking boundary.
//!
//! Keep socket setup and transport state in this module. Gameplay should keep
//! consuming frame inputs, and rendering shells should keep owning display state.

use std::net::SocketAddr;

/// Socket address used by future host/connect code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkAddress {
    socket_addr: SocketAddr,
}

impl NetworkAddress {
    /// Wraps a socket address for use by networking code.
    #[must_use]
    pub const fn new(socket_addr: SocketAddr) -> Self {
        Self { socket_addr }
    }

    /// Returns the underlying socket address.
    #[must_use]
    pub const fn socket_addr(self) -> SocketAddr {
        self.socket_addr
    }
}
