// 02_data_processing.fx
// Demonstrates File I/O, JSON processing, and data manipulation.

let fs = import("std:fs")
let json = import("std:json")
let time = import("std:time")

print("--- 02: Data Processing ---")
let start_time = time.now_ms()

// Generate a JSON payload
let mock_data = "[\\{\"id\":101,\"amount\":150.75\\},\\{\"id\":102,\"amount\":400.0\\},\\{\"id\":103,\"amount\":50.25\\}]"
let filename = "/tmp/sales_data.json"

// Write to file
fs.write_file_or_throw(filename, mock_data)
print("Data written to " + filename)

// Read and parse
let content = fs.read_file_or_throw(filename)
let sales = json.parse(content)

var total_revenue = 0.0
var i = 0
while i < len(sales) {
    let sale = sales[i]
    total_revenue = total_revenue + sale.amount
    i = i + 1
}

print("Total Revenue: $" + total_revenue)

// Cleanup
fs.remove_file(filename)
let elapsed = time.now_ms() - start_time
print("Processing completed in " + elapsed + "ms.")
