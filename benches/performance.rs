use criterion::{black_box, criterion_group, criterion_main, Criterion};
use omni::*;
use std::time::Duration;

fn benchmark_brain_creation(c: &mut Criterion) {
    c.bench_function("brain_creation", |b| {
        b.iter(|| black_box(brain::OmniBrain::new()))
    });
}

fn benchmark_config_loading(c: &mut Criterion) {
    c.bench_function("config_loading", |b| {
        b.iter(|| black_box(config::OmniConfig::default()))
    });
}

fn benchmark_search_engine(c: &mut Criterion) {
    c.bench_function("search_engine_creation", |b| {
        b.iter(|| black_box(search::SearchEngine::new()))
    });
}

fn benchmark_manifest_parsing(c: &mut Criterion) {
    let manifest_content = r#"
name: "Benchmark Test"
description: "Performance test manifest"
packages:
  - name: "vim"
    source: "apt"
  - name: "git"
    source: "apt"
  - name: "firefox"
    source: "snap"
"#;

    c.bench_function("manifest_parsing", |b| {
        b.iter(|| {
            // Parse YAML content directly for benchmarking
            black_box(serde_yaml::from_str::<serde_yaml::Value>(manifest_content))
        })
    });
}

fn benchmark_dependency_resolution(c: &mut Criterion) {
    c.bench_function("dependency_resolver", |b| {
        b.iter(|| {
            let resolver = black_box(resolver::DependencyResolver::new());
            resolver
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        benchmark_brain_creation,
        benchmark_config_loading,
        benchmark_search_engine,
        benchmark_manifest_parsing,
        benchmark_dependency_resolution
);

criterion_main!(benches);
