use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub enum PoolError {
    EmptyPool,
}

pub struct ServerPool {
    servers: Vec<SocketAddr>,
    next: AtomicUsize,
}

impl ServerPool {
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        Self {
            servers,
            next: AtomicUsize::new(0),
        }
    }

    pub fn select(&self) -> Result<SocketAddr, PoolError> {
        if self.servers.is_empty() {
            return Err(PoolError::EmptyPool);
        }

        let server_count = self.servers.len();

        let index = self
            .next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(if current + 1 == server_count {
                    0
                } else {
                    current + 1
                })
            })
            .expect("selecetion update always return a next index");

        Ok(self.servers[index])
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::{PoolError, ServerPool};

    fn server(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    #[test]
    fn 서버를_입력_순서대로_선택한다() {
        // Given
        let first = server(9000);
        let second = server(9001);
        let third = server(9002);
        let pool = ServerPool::new(vec![first, second, third]);

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
        let pool = ServerPool::new(vec![first, second]);

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
        let pool = ServerPool::new(Vec::new());

        // When
        let result = pool.select();

        // Then
        assert!(matches!(result, Err(PoolError::EmptyPool)));
    }
}
