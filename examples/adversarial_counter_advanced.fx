let topia = import("topia")
let app = topia.App("Advanced Topia Counter", 500, 400)
var count = 0
var step = 1
var status = "Normal"
let btn_dec = topia.Button("- Step", func() {
    count = count - step
    status = "Decremented"
})
let btn_inc = topia.Button("+ Step", func() {
    count = count + step
    status = "Incremented"
})
let btn_step_up = topia.Button("Step +1", func() {
    step = step + 1
    status = "Step Increased"
})
let btn_step_down = topia.Button("Step -1", func() {
    if step > 1 {
        step = step - 1
        status = "Step Decreased"
    } else {
        status = "Step Minimum Reached"
    }
})
let btn_double = topia.Button("Double (*2)", func() {
    count = count * 2
    status = "Doubled"
})
let btn_reset = topia.Button("Reset All", func() {
    count = 0
    step = 1
    status = "Reset to Default"
})
let view = func() {
    topia.VStack([topia.Text("=== Advanced Reactive Counter ==="), topia.Text("Current Count: " + count), topia.Text("Step Size: " + step), topia.Text("Status: " + status), topia.HStack([btn_dec, btn_inc], 10), topia.HStack([btn_step_down, btn_step_up], 10), topia.HStack([btn_double, btn_reset], 10)], 8)
}
app.run(view)
