use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(super) struct RoundRobin {
    next: AtomicUsize,
}

impl RoundRobin {
    pub(super) const fn new() -> Self {
        Self {
            next: AtomicUsize::new(0),
        }
    }

    pub(super) fn select(&self, server_count: NonZeroUsize) -> usize {
        let server_count = server_count.get();
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
                Ok(_) => return index,
                Err(current_index) => index = current_index,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, sync::atomic::Ordering};

    use super::RoundRobin;

    #[test]
    fn 서버를_한_바퀴_선택하면_다음_인덱스가_처음으로_돌아간다() {
        // Given
        let round_robin = RoundRobin::new();
        let server_count = NonZeroUsize::new(3).expect("server count should be non-zero");

        // When
        for _ in 0..server_count.get() {
            let _ = round_robin.select(server_count);
        }

        // Then
        assert_eq!(round_robin.next.load(Ordering::Relaxed), 0);
    }
}
