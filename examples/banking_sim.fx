struct Account {
    id: Int,
    name: String,
    balance: Float,
}

var db = [
    Account(101, "Alice", 500.0),
    Account(102, "Bob", 120.0)
]

let transfer = func(from_id, to_id, amount) {
    var from_idx = -1
    var to_idx = -1
    
    var i = 0
    while i < len(db) {
        if db[i].id == from_id { from_idx = i }
        if db[i].id == to_id { to_idx = i }
        i = i + 1
    }
    
    if from_idx == -1 {
        print("Error: Sender " + from_id + " not found.")
        return false
    }
    if to_idx == -1 {
        print("Error: Receiver " + to_id + " not found.")
        return false
    }
    
    if db[from_idx].balance < amount {
        print("Error: Insufficient funds for " + db[from_idx].name)
        return false
    }
    
    db[from_idx].balance = db[from_idx].balance - amount
    db[to_idx].balance = db[to_idx].balance + amount
    print("Success: Transferred $" + amount + " from " + db[from_idx].name + " to " + db[to_idx].name)
    return true
}

let print_balances = func() {
    print("--- Current Balances ---")
    var i = 0
    while i < len(db) {
        print(db[i].name + ": $" + db[i].balance)
        i = i + 1
    }
    print("------------------------")
}

print_balances()
transfer(101, 102, 150.0)
transfer(102, 101, 500.0) // should fail
transfer(102, 103, 10.0) // should fail
transfer(102, 101, 50.0)
print_balances()
