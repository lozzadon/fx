# `f(x)` Language Syntax Guide

Welcome to the official syntax guide for **`f(x)`**! 

`f(x)` is a dynamically-typed, interpreted language built in Rust. Its syntax is focusing on clean, readable code with powerful functional programming capabilities.

---

## 1. Variables

You can declare variables using `let` (traditionally for constants, though currently acts as a standard binder) or `var`.

```fx
let greeting = "Hello!"
var score = 100

// Variables can be reassigned:
score = 150
```

---

## 2. Data Types

`f(x)` currently supports four primitive data types.

### Integers
Standard 64-bit integers for mathematical operations.
```fx
let age = 25
let negative = -10
```

### Booleans
Standard truth values.
```fx
let is_active = true
let is_game_over = false
```

### Strings
Text data wrapped in double quotes.
```fx
let message = "Welcome to f(x)"
let combined = "Hello " + "World" // "Hello World"
```

### Arrays
Ordered lists of expressions. Arrays can contain mixed types and even dynamic expressions that are evaluated on the fly.
```fx
let list = [1, 2, "three", true]
let math_list = [1 + 1, 10 * 10] // Evaluates to [2, 100]

// Access elements using a 0-based index:
let first = list[0] // 1
```

---

## 3. Math & Operators

`f(x)` understands standard operator precedence (PEMDAS). Multiplication and division evaluate before addition and subtraction.

```fx
let math = 10 + 5 * 4 // Evaluates to 30, not 60!
let grouped = (10 + 5) * 4 // Evaluates to 60

// Supported Arithmetic: +, -, *, /
// Supported Comparison: <, >, ==, !=
```

---

## 4. Control Flow (If / Else)

You can branch logic using `if` and `else` statements. The condition does not need to be wrapped in parentheses.

```fx
let hp = 0

if hp < 1 {
    let status = "Dead"
} else {
    let status = "Alive"
}
```

---

## 5. Functions & Closures

Functions are first-class citizens in `f(x)`. You can define them using the `func` keyword.

### Named Functions
```fx
func add(a, b) {
    return a + b
}

let result = add(5, 10) // 15
```

### Optional Type Annotations
To maintain clean aesthetics, `f(x)` allows you to include type annotations. Currently, the interpreter will gracefully ignore them, allowing you to write highly readable code!

```fx
func multiply(x: Int, y: Int) -> Int {
    return x * y
}
```

### Closures (Anonymous Functions)
Because functions are treated as standard expressions, you can create them without names and pass them around like variables.

```fx
let get_multiplier = func(x) {
    return func(y) {
        return x * y
    }
}

let times_five = get_multiplier(5)
times_five(10) // 50
```

### Lexical Scoping
Functions remember the environment they were created in. If a closure references a variable from an outer scope, it will safely "capture" it!

```fx
let base = 100
func add_to_base(x) {
    return base + x 
}
add_to_base(50) // 150
```

### Dictionaries & Array Utilities
`f(x)` supports JavaScript-like dictionary mapping and functional array methods.
````fx
let user = {"name": "Alice", "score": 95}
let arr = [1, 2, 3]
let arr2 = push(arr, 4)

let evens = filter(arr2, func(x) { x == 2 || x == 4 })
````

### String Interpolation
Embed variables directly in your strings.
````fx
let name = "Alice"
print("Hello {name}, your score is {user["score"]}!")
````

### Pattern Matching (`match`)
Clean branching logic.
````fx
let code = 404
let status = match code {
    200 => "OK",
    404 => "Not Found",
    _ => "Unknown Error"
}
````

### Runtime Type Checking
Optionally enforce types in your function signatures to catch bugs at execution.
````fx
func safe_add(a: Int, b: Int) -> Int {
    a + b
}
// safe_add(1, "str") will throw a fatal runtime type mismatch!
````

### Try / Catch / Throw
Gracefully handle failures.
````fx
let result = try {
    throw "Server Disconnected!"
    "Success"
} catch e {
    "Failed with: {e}"
}
````

### Modules & Imports
Organize your code across files using the built-in `import` function.
````fx
// math.fx
func add(a, b) { a + b }
let pi = 3.14

// main.fx
let math = import("math.fx")
print(math["add"](10, 5))
print(math["pi"])
````
