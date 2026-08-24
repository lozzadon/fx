let ui = import("std:topia")

var count1 = 0
var count2 = 0
var count3 = 0

let render = func() {
    let total = count1 + count2 + count3
    
    let btn1 = ui.Button("Counter 1: " + count1, func() { count1 = count1 + 1 })
    let btn2 = ui.Button("Counter 2: " + count2, func() { count2 = count2 + 1 })
    let btn3 = ui.Button("Counter 3: " + count3, func() { count3 = count3 + 1 })
    
    let reset = ui.Button("Reset All", func() {
        count1 = 0
        count2 = 0
        count3 = 0
    })
    
    ui.VStack([
        ui.Text("f(x) Counter Dashboard", {"size": 28, "bold": true}),
        ui.HStack([btn1, btn2, btn3]),
        ui.Text("Total Sum: " + total, {"size": 20}),
        reset
    ])
}

let app = ui.App("Dashboard", 500, 300)
ui.run(app, render)
