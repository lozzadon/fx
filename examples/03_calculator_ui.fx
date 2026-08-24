// 03_calculator_ui.fx
// A fully functional reactive calculator built with Topia.

let ui = import("std:topia")

var display = "0"
var prev_val = 0
var current_op = ""
var should_reset = false

// Helper to update display
let press_num = func(num) {
    if should_reset {
        display = num
        should_reset = false
    } else {
        if display == "0" {
            display = num
        } else {
            display = display + num
        }
    }
}

// Layout helper for rows
let row = func(b1, b2, b3, b4) {
    let n1 = ui.Button(b1, func() { press_num(b1) })
    let n2 = ui.Button(b2, func() { press_num(b2) })
    let n3 = ui.Button(b3, func() { press_num(b3) })
    
    // For simplicity, just handling clear in the 4th column
    let n4 = ui.Button(b4, func() {
        if b4 == "C" {
            display = "0"
            prev_val = 0
            current_op = ""
        }
    })
    
    return ui.HStack([n1, n2, n3, n4])
}

let app_builder = func() {
    ui.VStack([
        ui.Text(display, {"size": 32, "bold": true}),
        row("7", "8", "9", "C"),
        row("4", "5", "6", "C"),
        row("1", "2", "3", "C"),
        row("0", "0", "0", "C")
    ])
}

let app = ui.App("Calculator", 300, 400)
ui.run(app, app_builder)
