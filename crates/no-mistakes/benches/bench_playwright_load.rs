use criterion::{criterion_group, criterion_main, Criterion};
use no_mistakes::playwright::playwright_config::load_many;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::hint::black_box;

fn bench_load_many(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    let mut config_paths = Vec::new();
    for i in 0..100 {
        let config_path = root.join(format!("playwright.config.{}.ts", i));
        fs::write(&config_path, format!(r#"
            export default {{
                name: 'config_{}',
                testDir: './tests',
            }};
        "#, i)).unwrap();
        config_paths.push(config_path);
    }

    c.bench_function("playwright_config_load_many_100", |b| {
        b.iter(|| {
            let _ = load_many(black_box(root), black_box(&config_paths), None);
        });
    });
}

criterion_group!(benches, bench_load_many);
criterion_main!(benches);
