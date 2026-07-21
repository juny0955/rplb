use std::{
    net::SocketAddr,
    num::{NonZeroU32, NonZeroUsize},
};

mod round_robin;

use round_robin::RoundRobin;

#[derive(Debug)]
pub enum PoolError {
    EmptyPool,
}

pub enum LoadBalancingPolicy {
    RR,   // RoundRobin
    SWRR, // SmoothWeightedRoundRobin
}

enum Selector {
    RoundRobin(RoundRobin),
    SmoothWeightedRoundRobin,
}

impl Selector {
    fn new(policy: LoadBalancingPolicy) -> Self {
        match policy {
            LoadBalancingPolicy::RR => Self::RoundRobin(RoundRobin::new()),
            LoadBalancingPolicy::SWRR => Self::SmoothWeightedRoundRobin,
        }
    }

    fn select(&self, server_count: NonZeroUsize) -> Result<usize, PoolError> {
        match self {
            Self::RoundRobin(round_robin) => Ok(round_robin.select(server_count)),
            Self::SmoothWeightedRoundRobin => unreachable!(),
        }
    }
}

pub struct Backend {
    addr: SocketAddr,
    weight: NonZeroU32,
}

impl Backend {
    pub const fn new(addr: SocketAddr, weight: NonZeroU32) -> Self {
        Self { addr, weight }
    }

    pub const fn weight(&self) -> NonZeroU32 {
        self.weight
    }
}

pub struct ServerPool {
    servers: Vec<Backend>,
    selector: Selector,
}

impl ServerPool {
    pub fn new(servers: Vec<Backend>, policy: LoadBalancingPolicy) -> Self {
        Self {
            servers,
            selector: Selector::new(policy),
        }
    }

    pub fn select(&self) -> Result<SocketAddr, PoolError> {
        let Some(server_count) = NonZeroUsize::new(self.servers.len()) else {
            return Err(PoolError::EmptyPool);
        };

        let index = self.selector.select(server_count)?;
        Ok(self.servers[index].addr)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, num::NonZeroU32};

    use super::{Backend, LoadBalancingPolicy, PoolError, ServerPool};

    fn server(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn backend(addr: SocketAddr) -> Backend {
        Backend::new(addr, NonZeroU32::MIN)
    }

    #[test]
    fn 서버를_입력_순서대로_선택한다() {
        // Given
        let first = server(9000);
        let second = server(9001);
        let third = server(9002);
        let pool = ServerPool::new(
            vec![backend(first), backend(second), backend(third)],
            LoadBalancingPolicy::RR,
        );

        // When
        let selected = [
            pool.select().expect("first server should be selected"),
            pool.select().expect("second server should be selected"),
            pool.select().expect("third server should be selected"),
        ];

        // Then
        assert_eq!(selected, [first, second, third]);
    }

    #[test]
    fn 마지막_서버_다음에는_첫_서버를_선택한다() {
        // Given
        let first = server(9000);
        let second = server(9001);
        let pool = ServerPool::new(
            vec![backend(first), backend(second)],
            LoadBalancingPolicy::RR,
        );

        // When
        let selected = [
            pool.select().expect("first server should be selected"),
            pool.select().expect("second server should be selected"),
            pool.select()
                .expect("first server should be selected again"),
        ];

        // Then
        assert_eq!(selected, [first, second, first]);
    }

    #[test]
    fn 서버가_없으면_빈_풀_오류를_반환한다() {
        // Given
        let pool = ServerPool::new(Vec::new(), LoadBalancingPolicy::RR);

        // When
        let result = pool.select();

        // Then
        assert!(matches!(result, Err(PoolError::EmptyPool)));
    }
}
