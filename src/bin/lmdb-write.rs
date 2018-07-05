#![feature(test)]

extern crate lmdb;
extern crate tempdir;

use lmdb::{
    Environment,
    Error,
    Transaction,
    WriteFlags,
};
use tempdir::TempDir;

fn main() {
    let dir = TempDir::new("test").unwrap();
    let env = Environment::new().open(dir.path()).unwrap();
    let db = env.open_db(None).unwrap();

    let mut txn = env.begin_rw_txn().unwrap();
    txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
    txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
    txn.put(db, b"key3", b"val3", WriteFlags::empty()).unwrap();
    txn.commit().unwrap();

    let mut txn = env.begin_rw_txn().unwrap();
    assert_eq!(b"val1", txn.get(db, b"key1").unwrap());
    assert_eq!(b"val2", txn.get(db, b"key2").unwrap());
    assert_eq!(b"val3", txn.get(db, b"key3").unwrap());
    assert_eq!(txn.get(db, b"key"), Err(Error::NotFound));

    txn.del(db, b"key1", None).unwrap();
    assert_eq!(txn.get(db, b"key1"), Err(Error::NotFound));
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;

    use lmdb::{
        Environment,
        Transaction,
        WriteFlags,
    };
    use tempdir::TempDir;

    use self::rand::{Rng, XorShiftRng};
    use self::test::{Bencher, black_box};

    pub fn get_key(n: u32) -> String {
        format!("key{}", n)
    }

    pub fn get_data(n: u32) -> String {
        format!("data{}", n)
    }

    pub fn setup_bench_db<'a>(num_rows: u32) -> (TempDir, Environment) {
        let dir = TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path()).unwrap();

        {
            let db = env.open_db(None).unwrap();
            let mut txn = env.begin_rw_txn().unwrap();
            for i in 0..num_rows {
                txn.put(db, &get_key(i), &get_data(i), WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        }
        (dir, env)
    }

    #[bench]
    fn bench_setup_bench_db(b: &mut Bencher) {
        let n = 100u32;
        b.iter(|| {
            let (_dir, _env) = setup_bench_db(n);
        });
    }

    #[bench]
    fn bench_get_rand(b: &mut Bencher) {
        let n = 100u32;
        let (_dir, env) = setup_bench_db(n);
        let db = env.open_db(None).unwrap();
        let txn = env.begin_ro_txn().unwrap();

        let mut keys: Vec<String> = (0..n).map(|n| get_key(n)).collect();
        XorShiftRng::new_unseeded().shuffle(&mut keys[..]);

        b.iter(|| {
            let mut i = 0usize;
            for key in &keys {
                i = i + txn.get(db, key).unwrap().len();
            }
            black_box(i);
        });
    }

}
