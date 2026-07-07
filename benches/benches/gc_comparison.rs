// gc_comparison.rs
// Compares boa_gc performance against oscars' MarkSweepGarbageCollector

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::{Path, PathBuf};
use std::hint::black_box;

// --- Boa Engine Setup ---
fn run_js_script(script_path: &Path) {
    let source = std::fs::read_to_string(script_path).expect("Failed to read script");
    let mut context = boa_engine::Context::default();
    
    // Parse and evaluate
    let _ = context.eval(boa_engine::Source::from_bytes(&source)).unwrap();
    
    // Call the `main` function to match bench script patterns
    let global_object = context.global_object();
    if let Ok(main_fn) = global_object.get("main", &mut context) {
        if let Some(main_obj) = main_fn.as_callable() {
            let _ = main_obj.call(&boa_engine::JsValue::undefined(), &[], &mut context).unwrap();
        }
    }
}

fn bench_scripts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts")
}

// --- Oscars GC Setup ---
use oscars::mark_sweep::{Gc, GcRefCell, Trace, Finalize, TraceColor, MarkSweepGarbageCollector};
use std::collections::HashMap;

#[derive(Clone, Trace, Finalize)]
struct JsValue {
    #[trace]
    obj: Option<Gc<GcRefCell<JsObject>>>,
    #[trace]
    closure: Option<Gc<JsClosure>>,
    data: f64,
}

#[derive(Clone, Trace, Finalize)]
struct JsObject {
    #[trace]
    properties: HashMap<String, JsValue>,
    #[trace]
    prototype: Option<Gc<GcRefCell<JsObject>>>,
}

#[derive(Clone, Trace, Finalize)]
struct JsClosure {
    #[trace]
    captured: Vec<JsValue>,
}

// --- Benchmark Group: Closures ---
fn bench_closures(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_closures");

    // 1. Boa side
    let script_path = bench_scripts_dir().join("closures").join("create.js");
    group.bench_function("boa_gc_full_js", |b| {
        b.iter(|| run_js_script(&script_path));
    });

    // 2. Oscars side
    for size in [1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("oscars_mempool3", size), size, |b, &size| {
            b.iter(|| {
                let mut gc = MarkSweepGarbageCollector::default();
                let mut closures = Vec::with_capacity(size);
                for i in 0..size {
                    let captured = vec![JsValue {
                        obj: None,
                        closure: None,
                        data: i as f64,
                    }];
                    let closure = Gc::new_in(JsClosure { captured }, &gc);
                    closures.push(closure);
                }
                
                // simulate collection
                black_box(closures);
                gc.collect();
            });
        });
    }
    group.finish();
}

// --- Benchmark Group: Strings (Simulated) ---
fn bench_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_strings");

    let script_path = bench_scripts_dir().join("strings").join("concat.js");
    group.bench_function("boa_gc_full_js", |b| {
        b.iter(|| run_js_script(&script_path));
    });

    for size in [1000, 5000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("oscars_mempool3", size), size, |b, &size| {
            b.iter(|| {
                let mut gc = MarkSweepGarbageCollector::default();
                let mut objects = Vec::with_capacity(size);
                for i in 0..size {
                    let mut props = HashMap::new();
                    props.insert("val".to_string(), JsValue {
                        obj: None,
                        closure: None,
                        data: i as f64,
                    });
                    objects.push(Gc::new_in(GcRefCell::new(JsObject {
                        properties: props,
                        prototype: None,
                    }), &gc));
                }
                black_box(objects);
                gc.collect();
            });
        });
    }
    group.finish();
}

// --- Benchmark Group: Objects ---
fn bench_objects(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_objects");

    let script_path = bench_scripts_dir().join("properties").join("access.js");
    group.bench_function("boa_gc_full_js", |b| {
        b.iter(|| run_js_script(&script_path));
    });

    for size in [100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("oscars_mempool3", size), size, |b, &size| {
            b.iter(|| {
                let mut gc = MarkSweepGarbageCollector::default();
                let mut objects = Vec::with_capacity(size);
                for i in 0..size {
                    let mut props = HashMap::new();
                    for p in 0..5 {
                        props.insert(format!("prop_{}", p), JsValue {
                            obj: None,
                            closure: None,
                            data: (i * p) as f64,
                        });
                    }
                    objects.push(Gc::new_in(GcRefCell::new(JsObject {
                        properties: props,
                        prototype: None,
                    }), &gc));
                }
                black_box(objects);
                gc.collect();
            });
        });
    }
    group.finish();
}

// --- Benchmark Group: Nested Objects (Prototype Chain) ---
fn bench_nested_objects(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_nested_objects");

    let script_path = bench_scripts_dir().join("prototypes").join("chain.js");
    group.bench_function("boa_gc_full_js", |b| {
        b.iter(|| run_js_script(&script_path));
    });
    
    group.bench_function("oscars_mempool3", |b| {
        b.iter(|| {
            let mut gc = MarkSweepGarbageCollector::default();
            let mut current = None;
            for _ in 0..100 {
                let obj = Gc::new_in(GcRefCell::new(JsObject {
                    properties: HashMap::new(),
                    prototype: current,
                }), &gc);
                current = Some(obj);
            }
            black_box(current);
            gc.collect();
        });
    });

    group.finish();
}

// --- Benchmark Group: Splay Tree ---
fn bench_splay_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_splay_tree");

    let script_path = bench_scripts_dir().join("v8-benches").join("splay.js");
    group.bench_function("boa_gc_full_js", |b| {
        b.iter(|| run_js_script(&script_path));
    });

    group.finish();
}

// --- Benchmark Group: GC Stress ---
fn bench_gc_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("gc_stress");

    let script_path = bench_scripts_dir().join("gc_stress").join("gc_stress.js");
    group.bench_function("boa_gc_full_js", |b| {
        b.iter(|| run_js_script(&script_path));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_closures,
    bench_strings,
    bench_objects,
    bench_nested_objects,
    bench_splay_tree,
    bench_gc_stress
);

criterion_main!(benches);
