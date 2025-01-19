/*
   Copyright 2024 Maksym Medvied

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::collections::{HashMap, HashSet};

use rand::{self, distributions::DistString, Rng};
use rand_core::{RngCore, SeedableRng};
use rand_pcg;

use crate::{EntityId, Record, TagsAndAttributes};

pub struct TestRng {
    rng: rand_pcg::Pcg64Mcg,
}

impl TestRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand_pcg::Pcg64Mcg::seed_from_u64(seed),
        }
    }

    pub fn rand_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// The range: [0, end).
    pub fn rand_range(&mut self, end: u64) -> u64 {
        self.rng.gen_range(0..end)
    }

    pub fn rand_string(&mut self, size_max: usize) -> String {
        let size = self.rand_range(size_max as u64) as usize;
        rand::distributions::Alphanumeric.sample_string(&mut self.rng, size)
    }

    pub fn rand_vec_u8(&mut self, size_max: usize) -> Vec<u8> {
        let size = self.rand_range(size_max as u64) as usize;
        let mut v = vec![0; size];
        self.rng.fill(&mut v[..]);
        v
    }

    pub fn rand_u128(&mut self) -> u128 {
        self.rng.gen::<u128>()
    }
}

pub fn random_tags(test_rng: &mut TestRng) -> HashSet<String> {
    let nr_tags = (test_rng.rand_u64() % 16) as usize;
    std::iter::repeat_with(|| test_rng.rand_string(16))
        .take(nr_tags)
        .collect()
}

pub fn random_attrs(test_rng: &mut TestRng) -> HashMap<String, String> {
    let nr_attrs = (test_rng.rand_u64() % 16) as usize;
    std::iter::repeat_with(|| {
        (test_rng.rand_string(16), test_rng.rand_string(16))
    })
    .take(nr_attrs)
    .collect()
}

pub fn random_tags_and_attrs(test_rng: &mut TestRng) -> TagsAndAttributes {
    TagsAndAttributes {
        tags: random_tags(test_rng),
        attrs: random_attrs(test_rng),
    }
}

pub fn random_record(test_rng: &mut TestRng) -> Record {
    Record {
        ta: random_tags_and_attrs(test_rng),
        data: if test_rng.rand_range(8) == 0 {
            None
        } else {
            Some(test_rng.rand_vec_u8(0x100))
        },
    }
}

pub fn random_entity_id(test_rng: &mut TestRng) -> EntityId {
    EntityId {
        id: test_rng.rand_u64(),
    }
}
