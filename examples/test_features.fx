// ==============================================================================
// test_features.fx: Demonstration and Verification of Proposals 2, 4, and 7
// ==============================================================================

print("=== 1. Feature Demonstration: Proposal 2 (Compound Assignment & Relational Operators) ===")
var score = 100
print("Initial score: {score}")

score += 50
print("score += 50 -> {score}") // 150

score -= 20
print("score -= 20 -> {score}") // 130

score *= 2
print("score *= 2  -> {score}") // 260

score /= 4
print("score /= 4  -> {score}") // 65

score %= 9
print("score %= 9  -> {score}") // 2 (65 % 9)

let is_valid = score <= 10 && score >= 0 && score % 2 == 0
print("score <= 10 && score >= 0 && score % 2 == 0 -> {is_valid}")


print("\n=== 2. Feature Demonstration: Proposal 4 (Range Expressions & Numeric For-Loops) ===")
print("Exclusive range loop (0..5):")
var sum_excl = 0
for i in 0..5 {
    print("  Iteration i = {i}")
    sum_excl += i
}
print("Sum of 0..5: {sum_excl}") // 0 + 1 + 2 + 3 + 4 = 10

print("Inclusive range loop (1..=5):")
var fact = 1
for i in 1..=5 {
    fact *= i
}
print("5! = {fact}") // 1 * 2 * 3 * 4 * 5 = 120

print("Lexer lookahead verification for 0..10 without float corruption:")
var count = 0
for x in 0..10 {
    count += 1
}
print("Count in 0..10: {count}") // 10


print("\n=== 3. Feature Demonstration: Proposal 7 (String Escapes & Utilities) ===")
let multiline = "Line 1\nLine 2\tIndented with tab"
print("Escape sequences (\\n, \\t):")
print(multiline)

let quote_str = "She said, \"Welcome to f(x)!\""
print("Escaped quotes: {quote_str}")

let escaped_brace = "Literal brace syntax: \{name\}"
print("Escaped brace: {escaped_brace}")

let raw_data = "   apple, banana, cherry, date   "
let trimmed = trim(raw_data)
print("trim(\"{raw_data}\") -> \"{trimmed}\"")

let fruits = split(trimmed, ", ")
print("split result count: {len(fruits)}")
print("fruits[0]: {fruits[0]}, fruits[2]: {fruits[2]}")

let replaced = replace(trimmed, "banana", "blueberry")
print("replace banana -> blueberry: {replaced}")

let joined = join(fruits, " -> ")
print("join fruits: {joined}")

let has_cherry = contains(trimmed, "cherry")
print("contains 'cherry': {has_cherry}")

let starts_ap = starts_with(trimmed, "apple")
print("starts_with 'apple': {starts_ap}")

let ends_dt = ends_with(trimmed, "date")
print("ends_with 'date': {ends_dt}")

let upper_str = to_upper(fruits[0])
print("to_upper('apple'): {upper_str}")

let lower_str = to_lower("SHOUTING")
print("to_lower('SHOUTING'): {lower_str}")

let sub = substring("Hello World", 0, 5)
print("substring('Hello World', 0, 5): {sub}")

print("\n=== ALL FEATURE DEMONSTRATIONS COMPLETED SUCCESSFULLY ===")
