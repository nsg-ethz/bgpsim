// BgpSim: BGP Network Simulator written in Rust
// Copyright 2022-2025 Tibor Schneider <sctibor@ethz.ch>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "rand")]
use rand::{prelude::*, rngs::StdRng};

/// A function that samples a graph.
pub trait TopologySampler {
    /// Returns an iterator that describes links between nodes. A node is identified using a unique
    /// IDs that are smaller than [`Self::num_nodes`].
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)>;

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize;
}

impl TopologySampler for Vec<(usize, usize)> {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        self.iter().copied()
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.iter()
            .flat_map(|(a, b)| [*a, *b])
            .max()
            .map(|x| x + 1)
            .unwrap_or(0)
    }
}

impl TopologySampler for &[(usize, usize)] {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        self.iter().copied()
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.iter()
            .flat_map(|(a, b)| [*a, *b])
            .max()
            .map(|x| x + 1)
            .unwrap_or(0)
    }
}

/// A sampler to generate a complete graph.
#[derive(Debug, Clone, Copy)]
pub struct CompleteGraph(pub usize);

impl TopologySampler for CompleteGraph {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        (0..self.0).flat_map(|a| ((a + 1)..self.0).map(move |b| (a, b)))
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.0
    }
}

/// A sampler to generate a graph with `n` nodes. Two nodes are connected with probability `p`.
#[derive(Debug, Clone)]
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub struct GnpGraph<R> {
    n: usize,
    p: f64,
    rng: R,
}

#[cfg(feature = "rand")]
impl GnpGraph<ThreadRng> {
    /// Create a new GNP graph with the default RNG (from entropy).
    pub fn new(n: usize, p: f64) -> Self {
        Self {
            n,
            p,
            rng: thread_rng(),
        }
    }
}

#[cfg(feature = "rand")]
impl GnpGraph<StdRng> {
    /// Create a new GNP graph with a seeded RNG.
    pub fn seeded(seed: u64, n: usize, p: f64) -> Self {
        Self {
            n,
            p,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

#[cfg(feature = "rand")]
impl<R> GnpGraph<R> {
    /// Create a new GNP graph with the given RNG
    pub fn from_rng(rng: R, n: usize, p: f64) -> Self {
        Self { n, p, rng }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> TopologySampler for GnpGraph<R> {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        (0..self.n)
            .flat_map(|a| ((a + 1)..self.n).map(move |b| (a, b)))
            .filter(|_| self.rng.gen_bool(self.p))
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.n
    }
}

/// A sampler to generate a graph with `n` nodes and `m` edges, sampled at random.
#[derive(Debug, Clone)]
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub struct GnmGraph<R> {
    n: usize,
    m: usize,
    rng: R,
}

#[cfg(feature = "rand")]
impl GnmGraph<ThreadRng> {
    /// Create a new GNM graph with the default RNG (from entropy).
    pub fn new(n: usize, m: usize) -> Self {
        Self {
            n,
            m,
            rng: thread_rng(),
        }
    }
}

#[cfg(feature = "rand")]
impl GnmGraph<StdRng> {
    /// Create a new GNM graph with a seeded RNG.
    pub fn seeded(seed: u64, n: usize, m: usize) -> Self {
        Self {
            n,
            m,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

#[cfg(feature = "rand")]
impl<R> GnmGraph<R> {
    /// Create a new GNM graph with the given RNG
    pub fn from_rng(rng: R, n: usize, m: usize) -> Self {
        Self { n, m, rng }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> TopologySampler for GnmGraph<R> {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        let n = self.n;
        let m = self.m;

        let mut links = std::collections::HashSet::new();
        while links.len() < m {
            let i = self.rng.gen_range(0..n);
            let j = self.rng.gen_range(0..n);
            let (i, j) = if i < j { (i, j) } else { (j, i) };
            if i != j {
                links.insert((i, j));
            }
        }

        links
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.n
    }
}

/// Generate a random graph with `n` nodes. Then, place them randomly on a `dim`-dimensional
/// euclidean space, where each component is within the range `0.0` to `1.0`. Then, connect two
/// nodes if and only if their euclidean distance is less than `dist`.
#[derive(Debug, Clone)]
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub struct GeometricGraph<R> {
    n: usize,
    dim: usize,
    dist: f64,
    rng: R,
}

#[cfg(feature = "rand")]
impl GeometricGraph<ThreadRng> {
    /// Create a new geometric graph with the default RNG (from entropy).
    pub fn new(n: usize, dim: usize, dist: f64) -> Self {
        Self {
            n,
            dim,
            dist,
            rng: thread_rng(),
        }
    }
}

#[cfg(feature = "rand")]
impl GeometricGraph<StdRng> {
    /// Create a new geometric graph with a seeded RNG.
    pub fn seeded(seed: u64, n: usize, dim: usize, dist: f64) -> Self {
        Self {
            n,
            dim,
            dist,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

#[cfg(feature = "rand")]
impl<R> GeometricGraph<R> {
    /// Create a new geometric graph with the given RNG
    pub fn from_rng(rng: R, n: usize, dim: usize, dist: f64) -> Self {
        Self { n, dim, dist, rng }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> TopologySampler for GeometricGraph<R> {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        let n = self.n;
        let dim = self.dim;
        let positions = Vec::from_iter(
            (0..n).map(|_| Vec::from_iter((0..dim).map(|_| self.rng.gen_range(0.0..1.0)))),
        );
        // cache the square distance
        let dist2 = self.dist * self.dist;

        (1..n)
            .flat_map(|j| (0..j).map(move |i| (i, j)))
            .filter(move |(i, j)| {
                let pi = &positions[*i];
                let pj = &positions[*j];
                let distance: f64 = (0..dim).map(|x| (pi[x] - pj[x])).map(|x| x * x).sum();
                distance < dist2
            })
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.n
    }
}

/// Generate a random graph using Barabási-Albert preferential attachment. A complete graph with
/// `m` nodes is grown by attaching new nodes each with `m` edges that are preferentially
/// attached to existing nodes with high degree.
#[derive(Debug, Clone)]
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub struct BarabasiAlbertGraph<R> {
    n: usize,
    m: usize,
    rng: R,
}

#[cfg(feature = "rand")]
impl BarabasiAlbertGraph<ThreadRng> {
    /// Create a new geometric graph with the default RNG (from entropy).
    pub fn new(n: usize, m: usize) -> Self {
        Self {
            n,
            m,
            rng: thread_rng(),
        }
    }
}

#[cfg(feature = "rand")]
impl BarabasiAlbertGraph<StdRng> {
    /// Create a new geometric graph with a seeded RNG.
    pub fn seeded(seed: u64, n: usize, m: usize) -> Self {
        Self {
            n,
            m,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

#[cfg(feature = "rand")]
impl<R> BarabasiAlbertGraph<R> {
    /// Create a new geometric graph with the given RNG
    pub fn from_rng(rng: R, n: usize, m: usize) -> Self {
        Self { n, m, rng }
    }
}

#[cfg(feature = "rand")]
impl<R: RngCore> TopologySampler for BarabasiAlbertGraph<R> {
    fn sample(&mut self) -> impl IntoIterator<Item = (usize, usize)> {
        let n = self.n;
        let m = self.m;
        let rng = &mut self.rng;

        let mut degree = vec![0; n];

        let mut links: Vec<_> = (0..m)
            .flat_map(|a| ((a + 1)..m).map(move |b| (a, b)))
            .collect();

        // update all first `m` routers to have `m-1` neighbors
        for i in 0..m {
            degree[i] = m - 1;
        }

        // if n <= (m + 1), then just create a complete graph with n nodes.
        if n <= (m + 1) {
            return links;
        }

        for i in m..n {
            let mut added_edges: std::collections::BTreeSet<_> = Default::default();

            // add m edges
            for _ in 0..m {
                let p: Vec<_> = degree
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j < i) // only connect with already added nodes
                    .filter(|(j, _)| !added_edges.contains(j)) // only connect with new nodes
                    .flat_map(|(j, degree)| std::iter::repeat(j).take(*degree))
                    .collect();
                let j = p[rng.gen_range(0..p.len())];
                links.push((i, j));
                *(&mut degree[i]) += 1;
                *(&mut degree[j]) += 1;
                added_edges.insert(j);
            }
        }

        links
    }

    /// The number of nodes that must be created.
    fn num_nodes(&mut self) -> usize {
        self.n
    }
}
