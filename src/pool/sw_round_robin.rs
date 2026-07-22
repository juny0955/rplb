use std::sync::Mutex;

use crate::pool::Server;

pub(super) struct SmoothWeightedRoundRobin {
    current_weights: Mutex<Vec<i64>>,
    total_weight: i64,
}

impl SmoothWeightedRoundRobin {
    pub(super) fn new(servers: &[Server]) -> Self {
        let total_weight = servers
            .iter()
            .map(|server| i64::from(server.weight().get()))
            .sum();

        Self {
            current_weights: Mutex::new(vec![0; servers.len()]),
            total_weight,
        }
    }

    pub(super) fn select(&self, servers: &[Server]) -> Option<usize> {
        if servers.is_empty() {
            return None;
        }

        let mut current_weights = match self.current_weights.lock() {
            Ok(cur) => cur,
            Err(poisoned) => poisoned.into_inner(),
        };

        let mut selected_index = 0;

        for (index, server) in servers.iter().enumerate() {
            current_weights[index] += i64::from(server.weight().get());

            if current_weights[index] > current_weights[selected_index] {
                selected_index = index;
            }
        }

        current_weights[selected_index] -= self.total_weight;

        Some(selected_index)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, num::NonZeroU32};

    use crate::pool::Server;

    use super::SmoothWeightedRoundRobin;

    fn server(port: u16, weight: u32) -> Server {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let weight = NonZeroU32::new(weight).expect("weight should be non-zero");

        Server::new(addr, weight)
    }

    #[test]
    fn 가중치가_3대1이면_네번마다_3대1_순서로_선택한다() {
        // Given
        let servers = [server(9000, 3), server(9001, 1)];
        let swrr = SmoothWeightedRoundRobin::new(&servers);

        // When
        let selected = (0..8)
            .map(|_| swrr.select(&servers).expect("a server should be selected"))
            .collect::<Vec<_>>();

        // Then
        assert_eq!(selected, [0, 0, 1, 0, 0, 0, 1, 0]);
    }

    #[test]
    fn 동일한_가중치면_입력_순서대로_번갈아_선택한다() {
        // Given
        let servers = [server(9000, 1), server(9001, 1)];
        let swrr = SmoothWeightedRoundRobin::new(&servers);

        // When
        let selected = (0..4)
            .map(|_| swrr.select(&servers).expect("a server should be selected"))
            .collect::<Vec<_>>();

        // Then
        assert_eq!(selected, [0, 1, 0, 1]);
    }

    #[test]
    fn 서버가_하나면_가중치와_관계없이_항상_선택한다() {
        // Given
        let servers = [server(9000, 7)];
        let swrr = SmoothWeightedRoundRobin::new(&servers);

        // When
        let selected = (0..3)
            .map(|_| swrr.select(&servers).expect("a server should be selected"))
            .collect::<Vec<_>>();

        // Then
        assert_eq!(selected, [0, 0, 0]);
    }

    #[test]
    fn 서버가_없으면_선택하지_않는다() {
        // Given
        let servers = [];
        let swrr = SmoothWeightedRoundRobin::new(&servers);

        // When
        let selected = swrr.select(&servers);

        // Then
        assert_eq!(selected, None);
    }
}
