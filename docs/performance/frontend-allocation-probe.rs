// Reproduction asset for the allocation controls in docs/architecture.md.
// Linux only; copy into the crate examples directory in an isolated checkout.
use no_mistakes::benchmark_support::{
    collect_language_frontend_edges, collect_language_frontend_facts, language_frontend_fixture,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::sync::Mutex;

struct Probe;
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static WATCH: AtomicBool = AtomicBool::new(false);
static SAMPLES: Mutex<(usize, [[usize; 16]; 256])> = Mutex::new((0, [[0; 16]; 256]));
unsafe extern "C" {
    fn backtrace(buffer: *mut *mut c_void, size: i32) -> i32;
}

fn allocation(size: usize) {
    let call = CALLS.fetch_add(1, Relaxed);
    BYTES.fetch_add(size, Relaxed);
    let live = LIVE.fetch_add(size, Relaxed) + size;
    if WATCH.load(Relaxed) {
        PEAK.fetch_max(live, Relaxed);
        if call % 256 == 0 {
            let mut stack = [std::ptr::null_mut(); 16];
            unsafe {
                backtrace(stack.as_mut_ptr(), 16);
            }
            let mut samples = SAMPLES.lock().unwrap();
            let index = samples.0;
            if index < samples.1.len() {
                samples.1[index] = stack.map(|address| address as usize);
                samples.0 += 1;
            }
        }
    }
}

unsafe impl GlobalAlloc for Probe {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() {
            allocation(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc_zeroed(layout);
        if !pointer.is_null() {
            allocation(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        System.dealloc(pointer, layout);
    }
    unsafe fn realloc(&self, pointer: *mut u8, old: Layout, size: usize) -> *mut u8 {
        let result = System.realloc(pointer, old, size);
        if !result.is_null() {
            LIVE.fetch_sub(old.size(), Relaxed);
            allocation(size);
        }
        result
    }
}
#[global_allocator]
static ALLOCATOR: Probe = Probe;

fn measure(label: &str, mut run: impl FnMut()) {
    for iteration in 0..12 {
        SAMPLES.lock().unwrap().0 = 0;
        let baseline = LIVE.load(Relaxed);
        let calls = CALLS.load(Relaxed);
        let bytes = BYTES.load(Relaxed);
        PEAK.store(baseline, Relaxed);
        WATCH.store(true, Relaxed);
        run();
        WATCH.store(false, Relaxed);
        let calls = CALLS.load(Relaxed) - calls;
        let bytes = BYTES.load(Relaxed) - bytes;
        let extra_peak = PEAK.load(Relaxed).saturating_sub(baseline);
        println!(
            "MEASURE {label} {iteration} calls={calls} requested_bytes={bytes} peak_extra_bytes={extra_peak}"
        );
        if iteration == 0 {
            let samples = SAMPLES.lock().unwrap();
            for stack in &samples.1[..samples.0] {
                print!("STACK {label}");
                for &address in stack {
                    if address != 0 {
                        let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
                        if unsafe { libc::dladdr(address as *const c_void, &mut info) } != 0
                            && !info.dli_fname.is_null()
                        {
                            let module = unsafe { std::ffi::CStr::from_ptr(info.dli_fname) }
                                .to_string_lossy();
                            print!(" {}+{:#x}", module, address - info.dli_fbase as usize);
                        }
                    }
                }
                println!();
            }
        }
    }
}
fn main() {
    // Warm lazy unwinding and worker initialization outside every measured collector call.
    let mut stack = [std::ptr::null_mut(); 16];
    unsafe {
        backtrace(stack.as_mut_ptr(), 16);
    }
    rayon::broadcast(|_| {
        let mut stack = [std::ptr::null_mut(); 16];
        unsafe {
            backtrace(stack.as_mut_ptr(), 16);
        }
    });
    let fixture = language_frontend_fixture();
    assert_eq!(fixture.files.len(), 117);
    let facts = collect_language_frontend_facts(&fixture);
    let edges = collect_language_frontend_edges(&fixture);
    assert_eq!(facts.parsed_files, 69);
    assert_eq!(edges.edges, 125);
    // Calibration: one reserved allocation with exactly known size; it is dropped in phase.
    measure("calibration", || {
        let value = std::hint::black_box(Box::new([0u8; 4096]));
        drop(std::hint::black_box(value));
    });
    measure("extract", || {
        assert_eq!(
            std::hint::black_box(collect_language_frontend_facts(&fixture)),
            facts
        );
    });
    measure("edges", || {
        assert_eq!(
            std::hint::black_box(collect_language_frontend_edges(&fixture)),
            edges
        );
    });
}
