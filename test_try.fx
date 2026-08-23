let result = try {
    print("Doing something risky...")
    throw {"code": 500, "message": "Database disconnected"}
    print("This will not print")
    "Success"
} catch e {
    "Failed with error: {e}"
}
print("Result: {result}")
