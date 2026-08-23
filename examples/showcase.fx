let math = import("examples/math.fx")
let radius = 5
let area = math["pi"] * radius * radius
let user = {"name": "Alice", "scores": [85, 92, 100], "active": true}
let evens = filter([1, 2, 3, 4, 5], func(x) {
    x < 4
})
let doubled = map(evens, func(x) {
    x * 2
})
let name = user["name"]
print("Hello " + name + "! Your area is " + area)
let active = user["active"]
let status = match active {
    true => {
        "Online"
    },
    false => {
        "Offline"
    },
    _ => {
        "Unknown"
    },
}
let result = try {
    throw "An error occurred!"
    "Success"
} catch e {
    "Caught error: " + e
}
var sum = 0
for score in user["scores"] {
    var sum = sum + score
}
let safe_multiply = func safe_multiply(a: Int, b: Int) -> Int {
    a * b
}
let product = safe_multiply(math["add"](10, 5), 2)
print("Status: " + status)
print("Result: " + result)
print("Product: " + product)
