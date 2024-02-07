use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path;
use turbocopy::CopyIoUring;
use turbocopy::CopyLibc;
use turbocopy::TurboCopy;

use libc::stat64;
use std::{fs::File, mem, os::fd::AsRawFd, path::PathBuf};

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size");
    group.sample_size(10);

    let source: PathBuf = path::PathBuf::from("/home/alvaro/repos/turbocopy/test.mov");
    let destination: PathBuf = path::PathBuf::from("/home/alvaro/repos/turbocopy/test.mov_copy");
    let blocksize: usize = (get_blocksize(&source) * 32) as usize;

    group.bench_function("CopyLibc::copy_with", |b| {
        b.iter(|| {
            CopyLibc::copy_with(
                black_box(&source),
                black_box(&destination),
                black_box(blocksize),
            );
        })
    });

    group.bench_function("CopyIoUring::copy_with", |b| {
        b.iter(|| {
            CopyIoUring::copy_with(
                black_box(&source),
                black_box(&destination),
                black_box(blocksize),
            );
        })
    });
}

#[inline]
fn get_blocksize(source_path: &PathBuf) -> i64 {
    let from_file = File::open(source_path).unwrap();
    unsafe {
        let mut raw_stat = mem::zeroed::<stat64>();
        libc::fstat64(from_file.as_raw_fd(), &mut raw_stat as *mut _);
        raw_stat.st_blksize
    }
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
