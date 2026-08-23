let time = import("std:time")

print("=========================================================")
print("  f(x) Engine Stress Test")
print("=========================================================")

// 1. Recursive Fibonacci (CPU bound function calls)
let fib = func(n: Int) -> Int {
    if n <= 1 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

let t1 = time.now_ms()
let f_res = fib(25)
let t2 = time.now_ms()
print("1. Recursive Fibonacci(25): " + f_res + " [Took: " + (t2 - t1) + " ms]")

// 2. Large Array Allocation & Mutation
let t3 = time.now_ms()
var arr = []
for i in 0..10000 {
    arr = push(arr, i)
}
for i in 0..10000 {
    arr[i] = arr[i] * 2
}
let t4 = time.now_ms()
print("2. Array Allocation & Mutation (10,000 items) [Took: " + (t4 - t3) + " ms]")

// 3. Struct Instantiation & Field Access
struct Entity {
    id: Int,
    active: Bool,
    health: Int
}
let t5 = time.now_ms()
var entities = []
for i in 0..5000 {
    entities = push(entities, Entity(i, true, 100))
}
var active_count = 0
for i in 0..5000 {
    if entities[i].health > 0 {
        entities[i].health -= 10
        active_count += 1
    }
}
let t6 = time.now_ms()
print("3. Struct Instantiation & Mutation (5,000 items) [Took: " + (t6 - t5) + " ms]")

// 4. String Manipulation & JSON
let json = import("std:json")
let t7 = time.now_ms()
var s = ""
for i in 0..1000 {
    s += "A"
}
let payload = {"data": s, "count": 1000}
let serialized = json.stringify(payload)
let parsed = json.parse(serialized)
let t8 = time.now_ms()
print("4. String Concatenation & JSON Parse/Stringify [Took: " + (t8 - t7) + " ms]")

let total = t8 - t1
print("=========================================================")
print("  TOTAL TIME: " + total + " ms")
print("=========================================================")
