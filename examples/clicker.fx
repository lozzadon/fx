// clicker.fx
// An Idle Clicker game built in f(x)

let time = import("std:time")
let ui = import("std:topia")

// Game State
var score = 0
var click_power = 1
var auto_clickers = 0

// Store Costs
var upgrade_click_cost = 10
var auto_clicker_cost = 50

var last_sec = time.now_secs()

let click = func() {
    score = score + click_power
}

let buy_click_upgrade = func() {
    if score >= upgrade_click_cost {
        score = score - upgrade_click_cost
        click_power = click_power + 1
        // Increase cost by ~50% each time
        upgrade_click_cost = upgrade_click_cost + (upgrade_click_cost / 2) 
    }
}

let buy_auto_clicker = func() {
    if score >= auto_clicker_cost {
        score = score - auto_clicker_cost
        auto_clickers = auto_clickers + 1
        // Increase cost by ~33% each time
        auto_clicker_cost = auto_clicker_cost + (auto_clicker_cost / 3) 
    }
}

let render = func() {
    // Process passive income based on elapsed time in seconds
    let current_time = time.now_secs()
    if current_time > last_sec {
        let passed = current_time - last_sec
        score = score + (auto_clickers * passed)
        last_sec = current_time
    }
    
    ui.Center(
        ui.VStack([
            ui.Text("f(x) Clicker", {"size": 36, "bold": true}),
            ui.Empty(),
            
            ui.Text("Score: " + score, {"size": 64, "bold": true}),
            ui.Text("Passive Income: " + auto_clickers + " per sec", {"size": 18}),
            ui.Empty(),
            
            ui.Button("     CLICK ME! (+" + click_power + ")     ", click),
            ui.Empty(),
            ui.Empty(),
            
            ui.Text("Upgrades", {"size": 24, "bold": true}),
            ui.HStack([
                ui.Button("Upgrade Click (Cost: " + upgrade_click_cost + ")", buy_click_upgrade),
                ui.Button("Buy Auto-Clicker (Cost: " + auto_clicker_cost + ")", buy_auto_clicker)
            ], {"spacing": 20})
            
        ], {"spacing": 10})
    )
}

let app = ui.App("f(x) Clicker Game", 600, 500)
ui.run(app, render)
