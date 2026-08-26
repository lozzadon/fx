let result = match 5 {
    1 => "one",
    2 => "two",
    5 => "five",
    _ => "other"
}
print("match result: " + result)

try {
    print("in try block")
    throw "my custom error"
    print("this shouldn't print")
} catch e {
    print("caught error: " + e)
}
