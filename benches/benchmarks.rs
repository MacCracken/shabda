//! Criterion benchmarks for shabda G2P conversion.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use shabda::prelude::*;

fn bench_convert_hello_world(c: &mut Criterion) {
    c.bench_function("g2p_hello_world", |b| {
        let g2p = G2PEngine::new(Language::English);
        b.iter(|| {
            let events = g2p.convert("hello world").unwrap();
            black_box(events);
        });
    });
}

fn bench_convert_sentence(c: &mut Criterion) {
    c.bench_function("g2p_sentence", |b| {
        let g2p = G2PEngine::new(Language::English);
        b.iter(|| {
            let events = g2p
                .convert("the cat sat on the mat and was not happy")
                .unwrap();
            black_box(events);
        });
    });
}

fn bench_speak_hello(c: &mut Criterion) {
    c.bench_function("speak_hello", |b| {
        let g2p = G2PEngine::new(Language::English);
        let voice = svara::voice::VoiceProfile::new_male();
        b.iter(|| {
            let samples = g2p.speak("hello", &voice, 44100.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_speak_sentence(c: &mut Criterion) {
    c.bench_function("speak_sentence", |b| {
        let g2p = G2PEngine::new(Language::English);
        let voice = svara::voice::VoiceProfile::new_male();
        b.iter(|| {
            let samples = g2p
                .speak("the cat sat on the mat", &voice, 44100.0)
                .unwrap();
            black_box(samples);
        });
    });
}

criterion_group!(
    benches,
    bench_convert_hello_world,
    bench_convert_sentence,
    bench_speak_hello,
    bench_speak_sentence,
);

criterion_main!(benches);
