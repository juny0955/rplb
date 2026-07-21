use std::{
    net::SocketAddr,
    num::{NonZeroU32, NonZeroUsize},
};

mod round_robin;
mod sw_round_robin;

use round_robin::RoundRobin;

use crate::pool::sw_round_robin::SmoothWeightedRoundRobin;

#[derive(Debug)]
pub enum PoolError {
    EmptyPool,
}

pub enum LoadBalancingPolicy {
    RR,   // RoundRobin
    SWRR, // SmoothWeightedRoundRobin
}

enum Selector {
    RR(RoundRobin),
    Swrr(SmoothWeightedRoundRobin),
}

impl Selector {
    fn new(policy: LoadBalancingPolicy, servers: &[Server]) -> Self {
        match policy {
            LoadBalancingPolicy::RR => Self::RR(RoundRobin::new()),
            LoadBalancingPolicy::SWRR => Self::Swrr(SmoothWeightedRoundRobin::new(servers)),
        }
    }

    fn select(&self, servers: &[Server]) -> Result<usize, PoolError> {
        let Some(server_count) = NonZeroUsize::new(servers.len()) else {
            return Err(PoolError::EmptyPool);
        };

        match self {
            Self::RR(rr) => Ok(rr.select(server_count)),
            Self::Swrr(swrr) => swrr.select(servers).ok_or(PoolError::EmptyPool),
        }
    }
}

pub struct Server {
    addr: SocketAddr,
    weight: NonZeroU32,
}

impl Server {
    pub const fn new(addr: SocketAddr, weight: NonZeroU32) -> Self {
        Self { addr, weight }
    }

    pub const fn weight(&self) -> NonZeroU32 {
        self.weight
    }
}

pub struct ServerPool {
    servers: Vec<Server>,
    selector: Selector,
}

impl ServerPool {
    pub fn new(servers: Vec<Server>, policy: LoadBalancingPolicy) -> Self {
        let selector = Selector::new(policy, &servers);

        Self { servers, selector }
    }

    pub fn select(&self) -> Result<SocketAddr, PoolError> {
        let index = self.selector.select(&self.servers)?;
        Ok(self.servers[index].addr)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, num::NonZeroU32};

    use super::{LoadBalancingPolicy, PoolError, Server, ServerPool};

    fn server_address(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn server(addr: SocketAddr) -> Server {
        Server::new(addr, NonZeroU32::MIN)
    }

    #[test]
    fn 서버를_입력_순서대로_선택한다() {
        // Given
        let first = server_address(9000);
        let second = server_address(9001);
        let third = server_address(9002);
        let pool = ServerPool::new(
            vec![server(first), server(second), server(third)],
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
        let first = server_address(9000);
        let second = server_address(9001);
        let pool = ServerPool::new(vec![server(first), server(second)], LoadBalancingPolicy::RR);

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
