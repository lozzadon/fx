// fx-bench/index.fx
// A benchmarking toolkit for f(x)

let time = import("std:time")

func run(name, iterations, fn) {
    print("Benchmarking '" + name + "' for " + iterations + " iterations...")
    
    let total_start = time.now_ms()
    
    // We could track min/max, but doing array appends in a tight loop 
    // might skew the benchmark, so we'll just track total time for now.
    
    let i = 0
    while i < iterations {
        fn()
        i = i + 1
    }
    
    let total_end = time.now_ms()
    let elapsed = total_end - total_start
    let avg = elapsed / iterations
    
    print("--- Benchmark Results: " + name + " ---")
    print("Total time: " + elapsed + " ms")
    print("Average time: " + avg + " ms/iter")
    
    return {
        "elapsed_ms": elapsed,
        "average_ms": avg
    }
}

return {
    "run": run
}
