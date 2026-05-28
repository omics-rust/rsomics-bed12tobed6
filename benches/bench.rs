use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;

fn bench_bed12tobed6(c: &mut Criterion) {
    let bin = env!("CARGO_BIN_EXE_rsomics-bed12tobed6");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bed12 = manifest.join("tests/golden/input.bed12");
    c.bench_function("rsomics-bed12tobed6 golden", |b| {
        b.iter(|| {
            let out = Command::new(black_box(bin))
                .args(["-i", bed12.to_str().unwrap()])
                .output()
                .unwrap();
            assert!(out.status.success());
        });
    });
}

criterion_group!(benches, bench_bed12tobed6);
criterion_main!(benches);
