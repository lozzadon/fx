// fx-math/index.fx
// Official f(x) Advanced Math Package

let is_even = func(n) { return n % 2 == 0 }
let is_odd = func(n) { return n % 2 != 0 }

let clamp = func(val, min_val, max_val) {
    if val < min_val { return min_val }
    if val > max_val { return max_val }
    return val
}

// Maps a value from one range to another
let map_range = func(val, in_min, in_max, out_min, out_max) {
    return (val - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

// Factorial (iterative to avoid call stack limits)
let factorial = func(n) {
    if n <= 1 { return 1 }
    var result = 1
    var i = 2
    while i <= n {
        result = result * i
        i = i + 1
    }
    return result
}

// Fibonacci (iterative for O(n) performance)
let fibonacci = func(n) {
    if n <= 0 { return 0 }
    if n == 1 { return 1 }
    var a = 0
    var b = 1
    var i = 2
    while i <= n {
        var temp = a + b
        a = b
        b = temp
        i = i + 1
    }
    return b
}

// Greatest Common Divisor
let gcd = func(a, b) {
    var x = a
    var y = b
    while y != 0 {
        var temp = y
        y = x % y
        x = temp
    }
    return x
}

// Least Common Multiple
let lcm = func(a, b) {
    if a == 0 { return 0 }
    if b == 0 { return 0 }
    return (a * b) / gcd(a, b)
}

// Is Prime (Optimized O(sqrt(n)))
let is_prime = func(n) {
    if n <= 1 { return false }
    if n <= 3 { return true }
    if n % 2 == 0 { return false }
    if n % 3 == 0 { return false }
    var i = 5
    while i * i <= n {
        if n % i == 0 { return false }
        if n % (i + 2) == 0 { return false }
        i = i + 6
    }
    return true
}

let sum = func(arr) {
    var total = 0
    var i = 0
    while i < len(arr) {
        total = total + arr[i]
        i = i + 1
    }
    return total
}

let average = func(arr) {
    if len(arr) == 0 { return 0 }
    return sum(arr) / len(arr)
}

return {
    "is_even": is_even,
    "is_odd": is_odd,
    "clamp": clamp,
    "map_range": map_range,
    "factorial": factorial,
    "fibonacci": fibonacci,
    "gcd": gcd,
    "lcm": lcm,
    "is_prime": is_prime,
    "sum": sum,
    "average": average
}
