// 01_basics.fx
// A tour of f(x)'s core language features: Variables, Structs, Arrays, and Functions.

print("--- 01: Language Basics ---")

// Structs
struct Player {
    name: String,
    level: Int,
    is_active: Bool,
}

// Variables & Instantiation
var p1 = Player("Alice", 10, true)
var p2 = Player("Bob", 5, false)

// Arrays & Mutation
var party = [p1, p2]
print("Initial Party:")
print(party)

let level_up = func(player_idx, amount) {
    let p = party[player_idx]
    if p.is_active {
        party[player_idx].level = p.level + amount
        print(p.name + " leveled up to " + party[player_idx].level + "!")
    } else {
        print(p.name + " is inactive and cannot level up.")
    }
}

// Function calls
level_up(0, 5) // Alice
level_up(1, 2) // Bob

// Standard Library
let math = import("std:math")
print("Alice's Power Level: " + math.pow(party[0].level, 2))
