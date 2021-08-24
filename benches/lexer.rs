use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};

use rusty::*;

fn get_file(name: &str) -> String {
    let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    data_path.push("benches");
    data_path.push("data");
    data_path.push(name);

    assert!(data_path.exists());

    data_path.display().to_string()
}

fn repeat_file(name: &str, times: u32) -> String {
    let mut result = String::new();
    let content = get_file(name);

    for _ in 1..times {
        result.push_str(&content);
    }
    result
}

fn lex_small_file(c: &mut Criterion) {
    let content = repeat_file("lex.st", 1);
    let mut group = c.benchmark_group("Small");
    group.sample_size(1000);
    group.bench_function("lex small", |b| {
        b.iter(|| {
            let mut lexer = lex(&content);
            lexer.recover_until_close();
        });
    });
    group.finish();
}

fn lex_medium_file(c: &mut Criterion) {
    let content = repeat_file("lex.st", 50);
    c.bench_function("lex medium", |b| {
        b.iter(|| {
            let mut lexer = lex(&content);
            lexer.recover_until_close();
        });
    });
}

fn lex_large_file(c: &mut Criterion) {
    let content = repeat_file("lex.st", 5000);
    let mut group = c.benchmark_group("Large");
    group.sample_size(60);
    group.bench_function("lex large", |b| {
        b.iter(|| {
            let mut lexer = lex(&content);
            lexer.recover_until_close();
        });
    });
    group.finish();
}
criterion_group!(lexer, lex_small_file, lex_medium_file, lex_large_file);
criterion_main!(lexer);
