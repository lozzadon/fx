// f(x) Language Showcase

let math = import("examples/math.fx")

// 1. Floats, Integers, and Math
let radius = 5
let area = math["pi"] * (radius * radius)

// 2. Hash Maps and Arrays
let user = {
    "name": "Alice",
    "scores": [85, 92, 100],
    "active": true
}

// 3. Built-in Utilities (map, filter, reduce)
let evens = filter([1, 2, 3, 4, 5], func(x) { x % 2 == 0 })
let doubled = map(evens, func(x) { x * 2 })

// 4. String Interpolation
print("Hello {user["name"]}! Your area is {area}")

// 5. Pattern Matching
let status = match user["active"] {
    true => "Online",
    false => "Offline",
    _ => "Unknown"
}

// 6. Try / Catch
let result = try {
    throw "An error occurred!"
    "Success"
} catch e {
    "Caught error: {e}"
}

// 7. Iteration
var sum = 0
for score in user["scores"] {
    var sum = sum + score
}

// 8. Runtime Type Enforcement
func safe_multiply(a: Int, b: Int) -> Int {
    a * b
}
let product = safe_multiply(math["add"](10, 5), 2)

print("Status: {status}")
print("Result: {result}")
print("Product: {product}")
