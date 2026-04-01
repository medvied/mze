// SPDX-License-Identifier: AGPL-3.0-or-later
/*
    mze - personal knowledge database
    Copyright (C) 2024, 2025  Maksym Medvied

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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

pub fn random_tags(test_rng: &mut TestRng) -> Vec<String> {
    let nr_tags = (test_rng.rand_u64() % 16) as usize;
    std::iter::repeat_with(|| test_rng.rand_string(16))
        .take(nr_tags)
        .collect()
}

pub fn random_attributes(test_rng: &mut TestRng) -> Vec<(String, String)> {
    let nr_attributes = (test_rng.rand_u64() % 16) as usize;
    std::iter::repeat_with(|| {
        (test_rng.rand_string(16), test_rng.rand_string(16))
    })
    .take(nr_attributes)
    .collect()
}

pub fn random_tags_and_attributes(
    test_rng: &mut TestRng,
) -> TagsAndAttributes {
    TagsAndAttributes {
        tags: random_tags(test_rng),
        attributes: random_attributes(test_rng),
    }
}

pub fn random_record(test_rng: &mut TestRng) -> Record {
    Record {
        ta: random_tags_and_attributes(test_rng),
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
