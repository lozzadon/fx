let ui = import("std:topia")
let math = import("std:math")

var state = {
    "equation": "sin(x)",
    "min_x": -10.0,
    "max_x": 10.0,
    "points": 200.0,
    "error": ""
}

let render = func() {
    var points_arr = []
    var min_y = 1000000.0
    var max_y = -1000000.0
    
    var step = (state["max_x"] - state["min_x"]) / state["points"]
    var current_x = state["min_x"]
    var has_error = false
    
    while current_x <= state["max_x"] {
        var res = math.eval(state["equation"], current_x)
        if !res["ok"] {
            has_error = true
            state["error"] = res["err"]
            current_x = state["max_x"] + 1.0
        } else {
            var y_val = res["val"]
            if y_val < min_y { min_y = y_val }
            if y_val > max_y { max_y = y_val }
            points_arr = push(points_arr, [current_x, y_val])
            current_x = current_x + step
        }
    }
    
    if min_y > max_y {
        min_y = -10.0
        max_y = 10.0
    } else {
        var padding = (max_y - min_y) * 0.1
        if padding == 0.0 { padding = 1.0 }
        min_y = min_y - padding
        max_y = max_y + padding
    }
    
    var error_view = ui.Empty
    if has_error {
        error_view = ui.Text("Error: " + state["error"])
    } else {
        error_view = ui.Graph(points_arr, state["min_x"], state["max_x"], min_y, max_y)
    }

    ui.VStack([
        ui.Text("f(x) Graphing Calculator", {"size": 24.0, "bold": true}),
        ui.Text("Enter Equation (e.g., sin(x), x*x, cos(x)*PI):"),
        ui.TextInput(state["equation"], func(val) {
            state["equation"] = val
            state["error"] = ""
        }),
        error_view,
        ui.HStack([
            ui.Text("X Min:"),
            ui.Slider(state["min_x"], -50.0, 0.0, func(v) { state["min_x"] = v }),
            ui.Text("X Max:"),
            ui.Slider(state["max_x"], 0.0, 50.0, func(v) { state["max_x"] = v })
        ])
    ])
}

let app = ui.App("Graphing Calculator", 800, 600)
ui.run(app, render)
