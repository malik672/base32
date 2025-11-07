use base32::encoded_len;
use std::time::Instant;

fn main() {
    // Benchmark encoded_len for various input sizes
    let sizes = vec![1, 5, 10, 100, 1000, 10000, 100000, 1000000];

    println!("\n=== Encoded Length Benchmark ===\n");
    println!("{:<15} {:<15} {:<15}", "Input Size", "Encoded Len", "Time (µs)");
    println!("{:-<45}", "");

    for size in sizes {
        let start = Instant::now();

        // Run the function 10000 times to get measurable time
        for _ in 0..10000 {
            let _ = encoded_len(size);
        }

        let elapsed = start.elapsed();
        let encoded = encoded_len(size);
        let time_us = elapsed.as_micros() as f64 / 10000.0;

        println!("{:<15} {:<15} {:<15.3}", size, encoded, time_us);
    }

    println!();
}
