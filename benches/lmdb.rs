// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Portions of this code were copied or adapted from lmdb-rs
// <https://github.com/danburkert/lmdb-rs>, which is written by Dan Burkert.

#[macro_use]
extern crate criterion;
use criterion::Criterion;

extern crate lmdb;
extern crate tempdir;

extern crate rand;
extern crate walkdir;

use self::rand::{
    thread_rng,
    Rng,
};
use self::walkdir::WalkDir;
use lmdb::{
    Cursor,
    Environment,
    EnvironmentFlags,
    Transaction,
    WriteFlags,
};
use std::{
    thread,
    time,
};
use tempdir::TempDir;

const NUM_PAIRS: [u32; 3] = [1, 128, 1024];

fn get_key(n: u32) -> [u8; 4] {
    let b1: u8 = ((n >> 24) & 0xff) as u8;
    let b2: u8 = ((n >> 16) & 0xff) as u8;
    let b3: u8 = ((n >> 8) & 0xff) as u8;
    let b4: u8 = (n & 0xff) as u8;
    [b1, b2, b3, b4]
}

fn get_value(n: u32) -> Vec<u8> {
    format!("data{}", n).into_bytes()
}

fn get_pair(n: u32) -> ([u8; 4], Vec<u8>) {
    (get_key(n), get_value(n))
}

fn setup_bench_db(num_pairs: u32) -> (TempDir, Environment) {
    let dir = TempDir::new("test").unwrap();
    let env = Environment::new().open(dir.path()).unwrap();

    {
        let db = env.open_db(None).unwrap();
        let mut txn = env.begin_rw_txn().unwrap();
        for i in 0..num_pairs {
            txn.put(db, &get_key(i), &get_value(i), WriteFlags::empty()).unwrap();
        }
        txn.commit().unwrap();
    }

    (dir, env)
}

fn bench_open_db(c: &mut Criterion) {
    let dir = TempDir::new("bench_open_db").unwrap();

    // Create the database first so we only measure the time to open
    // an existing database.
    {
        let env = Environment::new().open(dir.path()).unwrap();
        let _db = env.open_db(None).unwrap();
    }

    c.bench_function("lmdb_open_db", move |b| {
        b.iter(|| {
            let env = Environment::new().open(dir.path()).unwrap();
            let _db = env.open_db(None).unwrap();
        })
    });
}

fn bench_put_seq_sync(c: &mut Criterion) {
    let dir = TempDir::new("test").unwrap();
    let env = Environment::new().open(dir.path()).unwrap();
    let db = env.open_db(None).unwrap();

    c.bench_function_over_inputs(
        "lmdb_put_seq_sync",
        move |b, &&num_pairs| {
            let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
            b.iter(|| {
                let mut txn = env.begin_rw_txn().unwrap();
                for (key, value) in &pairs {
                    txn.put(db, key, value, WriteFlags::empty()).unwrap();
                }
                txn.commit().unwrap();
            })
        },
        &NUM_PAIRS,
    );
}

fn bench_put_seq_async(c: &mut Criterion) {
    let dir = TempDir::new("test").unwrap();
    // LMDB writes are sync by default.  Set the MAP_ASYNC and WRITE_MAP
    // environment flags to make them async (along with using a writeable
    // memory map).
    let env = Environment::new()
        .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
        .open(dir.path())
        .unwrap();
    let db = env.open_db(None).unwrap();

    c.bench_function_over_inputs(
        "lmdb_put_seq_async",
        move |b, &&num_pairs| {
            let pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
            b.iter(|| {
                let mut txn = env.begin_rw_txn().unwrap();
                for (key, value) in &pairs {
                    txn.put(db, key, value, WriteFlags::empty()).unwrap();
                }
                txn.commit().unwrap();
            })
        },
        &NUM_PAIRS,
    );
}

fn bench_put_rand_sync(c: &mut Criterion) {
    let dir = TempDir::new("test").unwrap();
    let env = Environment::new().open(dir.path()).unwrap();
    let db = env.open_db(None).unwrap();

    c.bench_function_over_inputs(
        "lmdb_put_rand_sync",
        move |b, &&num_pairs| {
            let mut pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
            thread_rng().shuffle(&mut pairs[..]);
            b.iter(|| {
                let mut txn = env.begin_rw_txn().unwrap();
                for (key, value) in &pairs {
                    txn.put(db, key, value, WriteFlags::empty()).unwrap();
                }
                txn.commit().unwrap();
            })
        },
        &NUM_PAIRS,
    );
}

fn bench_put_rand_async(c: &mut Criterion) {
    let dir = TempDir::new("test").unwrap();
    // LMDB writes are sync by default.  Set the MAP_ASYNC and WRITE_MAP
    // environment flags to make them async (along with using a writeable
    // memory map).
    let env = Environment::new()
        .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
        .open(dir.path())
        .unwrap();
    let db = env.open_db(None).unwrap();

    c.bench_function_over_inputs(
        "lmdb_put_rand_async",
        move |b, &&num_pairs| {
            let mut pairs: Vec<([u8; 4], Vec<u8>)> = (0..num_pairs).map(|n| get_pair(n)).collect();
            thread_rng().shuffle(&mut pairs[..]);
            b.iter(|| {
                let mut txn = env.begin_rw_txn().unwrap();
                for (key, value) in &pairs {
                    txn.put(db, key, value, WriteFlags::empty()).unwrap();
                }
                txn.commit().unwrap();
            })
        },
        &NUM_PAIRS,
    );
}

fn bench_get_seq(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_get_seq",
        move |b, &&num_pairs| {
            let (_dir, env) = setup_bench_db(num_pairs);
            let db = env.open_db(None).unwrap();
            let keys: Vec<[u8; 4]> = (0..num_pairs).map(|n| get_key(n)).collect();
            b.iter(|| {
                let txn = env.begin_ro_txn().unwrap();
                let mut i = 0usize;
                for key in &keys {
                    i = i + txn.get(db, key).unwrap().len();
                }
            })
        },
        &NUM_PAIRS,
    );
}

fn bench_get_rand(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_get_rand",
        move |b, &&num_pairs| {
            let (_dir, env) = setup_bench_db(num_pairs);
            let db = env.open_db(None).unwrap();

            let mut keys: Vec<[u8; 4]> = (0..num_pairs).map(|n| get_key(n)).collect();
            thread_rng().shuffle(&mut keys[..]);

            let txn = env.begin_ro_txn().unwrap();
            b.iter(|| {
                let mut i = 0usize;
                for key in &keys {
                    i = i + txn.get(db, key).unwrap().len();
                }
            })
        },
        &NUM_PAIRS,
    );
}

/// Benchmark of iterator sequential read performance.
fn bench_get_seq_iter(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_get_seq_iter",
        move |b, &&num_pairs| {
            let (_dir, env) = setup_bench_db(num_pairs);
            let db = env.open_db(None).unwrap();

            let txn = env.begin_ro_txn().unwrap();
            b.iter(|| {
                let mut cursor = txn.open_ro_cursor(db).unwrap();
                let mut i = 0;
                let mut count = 0u32;

                for (key, data) in cursor.iter() {
                    i = i + key.len() + data.len();
                    count = count + 1;
                }

                assert_eq!(count, num_pairs);
            })
        },
        &NUM_PAIRS,
    );
}

// This measures space on disk, not time, reflecting the space taken
// by a database on disk into the time it takes the benchmark to complete.
// It is non-obvious to me that this is an accurate way to measure space,
// much less an optimal one.
fn bench_db_size(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "lmdb_db_size",
        move |b, &&num_pairs| {
            let (dir, _env) = setup_bench_db(num_pairs);
            let mut total_size = 0;

            for entry in WalkDir::new(dir.path()) {
                let metadata = entry.unwrap().metadata().unwrap();
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }

            b.iter(|| {
                // Convert size on disk to benchmark time by sleeping
                // for the total_size number of nanoseconds.
                thread::sleep(time::Duration::from_nanos(total_size));
            })
        },
        &NUM_PAIRS,
    );
}

criterion_group!(
    benches,
    bench_open_db,
    bench_put_seq_sync,
    bench_put_seq_async,
    bench_put_rand_sync,
    bench_put_rand_async,
    bench_get_seq,
    bench_get_rand,
    bench_get_seq_iter,
    bench_db_size,
);
criterion_main!(benches);
