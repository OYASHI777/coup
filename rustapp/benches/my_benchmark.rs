use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustapp::prob_manager::prob_state::ProbState;
fn contains_all_chars(string1: &str, string2: &str) -> bool {
    // This is faster
    let mut s1: String = string1.to_string();
    let mut s2: String = string2.to_string();

    for c in string2.chars() {
        if let Some(index) = s1.find(c) {
            s1.remove(index);
        } else {
            return false;
        }
    }
    true
}
use std::collections::HashMap;
fn contains_all_chars2(string1: &str, string2: &str) -> bool {
    let mut freqs1: HashMap<char, i32> = HashMap::new();
    let mut freqs2: HashMap<char, i32> = HashMap::new();

    // Count frequency of each character in string1
    for c in string1.chars() {
        *freqs1.entry(c).or_insert(0) += 1;
    }

    // Count frequency of each character in string2
    for c in string2.chars() {
        *freqs2.entry(c).or_insert(0) += 1;
    }

    // Check if string1 contains at least as many of each character as string2
    for (c, count) in freqs2 {
        if freqs1.get(&c).unwrap_or(&0) < &count {
            return false;
        }
    }

    true
}
fn benchmark_contains_all_chars(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Check");

    group.bench_function("contains_all_chars string search", |b| {
        b.iter(|| contains_all_chars(black_box("ABCDEFKJAKSBAKSB"), black_box("ASFJKBA")))
    });

    group.bench_function("contains_all_chars hashmap", |b| {
        b.iter(|| contains_all_chars2(black_box("ABCDEFKJAKSBAKSB"), black_box("ASFJKBA")))
    });

    group.finish();
}

fn bench_game_start(c: &mut Criterion) {
    c.bench_function("game_start", |b| {
        let mut state = ProbState::new(); // Setup test state
        b.iter(|| state.game_start("AB")); // Benchmark game_start
    });

    // c.bench_function("game_start_same", |b| {
    //     let mut state = ProbState::new(); // Setup test state
    //     b.iter(|| state.game_start_same()); // Benchmark game_start_rayon
    // });
    // c.bench_function("game_start_rayon", |b| {
    //     let mut state = ProbState::new(); // Setup test state
    //     b.iter(|| state.game_start_rayon()); // Benchmark game_start_rayon
    // });
}
fn bench_standard_move(c: &mut Criterion) {
    c.bench_function("standard_move", |b| {
        let mut state = ProbState::new(); // Setup test state
        state.game_start("AB");
        let mut hm: HashMap<String, f64> = HashMap::new();
        hm.insert("AA".to_string(), 1.0/15.0);
        hm.insert("AB".to_string(), 1.0/15.0);
        hm.insert("AC".to_string(), 1.0/15.0);
        hm.insert("AD".to_string(), 1.0/15.0);
        hm.insert("AE".to_string(), 1.0/15.0);
        hm.insert("BB".to_string(), 1.0/15.0);
        hm.insert("BC".to_string(), 1.0/15.0);
        hm.insert("BD".to_string(), 1.0/15.0);
        hm.insert("BE".to_string(), 1.0/15.0);
        hm.insert("CC".to_string(), 1.0/15.0);
        hm.insert("CD".to_string(), 1.0/15.0);
        hm.insert("CE".to_string(), 1.0/15.0);
        hm.insert("DD".to_string(), 1.0/15.0);
        hm.insert("DE".to_string(), 1.0/15.0);
        hm.insert("EE".to_string(), 1.0/15.0);
        b.iter(|| state.standard_move(&hm,2)); // Benchmark game_start
    });
    c.bench_function("standard_move_hp", |b| {
        let mut state = ProbState::new(); // Setup test state
        state.game_start("AB");
        let mut hm: HashMap<String, f64> = HashMap::new();
        hm.insert("AA".to_string(), 1.0/15.0);
        hm.insert("AB".to_string(), 1.0/15.0);
        hm.insert("AC".to_string(), 1.0/15.0);
        hm.insert("AD".to_string(), 1.0/15.0);
        hm.insert("AE".to_string(), 1.0/15.0);
        hm.insert("BB".to_string(), 1.0/15.0);
        hm.insert("BC".to_string(), 1.0/15.0);
        hm.insert("BD".to_string(), 1.0/15.0);
        hm.insert("BE".to_string(), 1.0/15.0);
        hm.insert("CC".to_string(), 1.0/15.0);
        hm.insert("CD".to_string(), 1.0/15.0);
        hm.insert("CE".to_string(), 1.0/15.0);
        hm.insert("DD".to_string(), 1.0/15.0);
        hm.insert("DE".to_string(), 1.0/15.0);
        hm.insert("EE".to_string(), 1.0/15.0);
        b.iter(|| state.standard_move_hp(&hm,2)); // Benchmark game_start
    });

    // c.bench_function("game_start_same", |b| {
    //     let mut state = ProbState::new(); // Setup test state
    //     b.iter(|| state.game_start_same()); // Benchmark game_start_rayon
    // });
    // c.bench_function("game_start_rayon", |b| {
    //     let mut state = ProbState::new(); // Setup test state
    //     b.iter(|| state.game_start_rayon()); // Benchmark game_start_rayon
    // });
}

// criterion_group!(benches, bench_standard_move);
// criterion_main!(benches);
