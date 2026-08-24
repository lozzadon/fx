let fs = import("std:fs")
let json = import("std:json")

print("--- Data Processor ---")

let mock_data = "[\\{\"name\":\"Alice\",\"age\":28,\"active\":true,\"balance\":1250\\},\\{\"name\":\"Bob\",\"age\":35,\"active\":false,\"balance\":400\\},\\{\"name\":\"Charlie\",\"age\":42,\"active\":true,\"balance\":3100\\}]"

let filename = "/tmp/mock_users.json"
fs.write_file_or_throw(filename, mock_data)

let content = fs.read_file_or_throw(filename)
let users = json.parse(content)

var active_count = 0
var total_balance = 0
var i = 0

while i < len(users) {
    let user = users[i]
    if user.active {
        active_count = active_count + 1
        total_balance = total_balance + user.balance
        print("Active User: " + user.name + " (Age: " + user.age + ") - Balance: $" + user.balance)
    }
    i = i + 1
}

if active_count > 0 {
    print("Total Active Users: " + active_count)
    print("Total Balance: $" + total_balance)
}

fs.remove_file(filename)
print("Data processing complete.")
