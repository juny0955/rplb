use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub enum PoolError {
    EmptyPool,
}

pub enum LoadBalancingPolicy {
    RR,   // RoundRobin
    SWRR, // SmoothWeightedRoundRobin
}

pub struct ServerPool {
    servers: Vec<SocketAddr>,
    policy: LoadBalancingPolicy,
    next: AtomicUsize,
}

impl ServerPool {
    pub fn new(servers: Vec<SocketAddr>, policy: LoadBalancingPolicy) -> Self {
        Self {
            servers,
            policy,
            next: AtomicUsize::new(0),
        }
    }

    pub fn select(&self) -> Result<SocketAddr, PoolError> {
        if self.servers.is_empty() {
            return Err(PoolError::EmptyPool);
        }

        match self.policy {
            LoadBalancingPolicy::RR => self.select_rr(),
            LoadBalancingPolicy::SWRR => self.select_swrr(),
        }
    }

    fn select_rr(&self) -> Result<SocketAddr, PoolError> {
        let server_count = self.servers.len();
        let mut index = self.next.load(Ordering::Relaxed);

        loop {
            let next_index = if index + 1 == server_count {
                0
            } else {
                index + 1
            };

            match self.next.compare_exchange_weak(
                index,
                next_index,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(self.servers[index]),
                Err(current_index) => index = current_index,
            }
        }
    }

    fn select_swrr(&self) -> Result<SocketAddr, PoolError> {
        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, sync::atomic::Ordering};

    use super::{LoadBalancingPolicy, PoolError, ServerPool};

    fn server(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    #[test]
    fn 서버를_입력_순서대로_선택한다() {
        // Given
        let first = server(9000);
        let second = server(9001);
        let third = server(9002);
        let pool = ServerPool::new(vec![first, second, third], LoadBalancingPolicy::RR);

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
        let pool = ServerPool::new(vec![first, second], LoadBalancingPolicy::RR);

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
    fn 서버를_한_바퀴_선택하면_다음_인덱스가_처음으로_돌아간다() {
        // Given
        let pool = ServerPool::new(
            vec![server(9000), server(9001), server(9002)],
            LoadBalancingPolicy::RR,
        );

        // When
        for _ in 0..3 {
            let _ = pool.select().expect("server should be selected");
        }

        // Then
        assert_eq!(pool.next.load(Ordering::Relaxed), 0);
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
