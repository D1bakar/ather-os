//! Deterministic pseudo-random helpers for host-only fuzz and property tests.
//!
//! Avoids external RNG crates so tests compile on Windows GNU toolchains without
//! `dlltool`.

/// Small fast PRNG (xorshift64*) for reproducible fuzz input.
#[derive(Clone, Copy, Debug)]
pub struct TestRng {
    state: u64,
}

impl TestRng {
    /// Creates an RNG seeded from `seed`.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    /// Returns the next u64.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Returns a value in `0..bound` (bound must be > 0).
    #[allow(dead_code)]
    pub fn next_bounded(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }

    /// Fills `buf` with pseudo-random bytes.
    #[allow(dead_code)]
    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let value = self.next_u64();
            let bytes = value.to_le_bytes();
            for (dst, src) in chunk.iter_mut().zip(bytes.iter()) {
                *dst = *src;
            }
        }
    }
}

/// Runs `cases` iterations of `body` with distinct RNG seeds.
pub fn for_each_case(cases: u32, body: impl Fn(&mut TestRng, u32)) {
    for case in 0..cases {
        let mut rng =
            TestRng::new(0xAE7E_0000_0000_0001 ^ u64::from(case).wrapping_mul(0x9E37_79B9));
        body(&mut rng, case);
    }
}
