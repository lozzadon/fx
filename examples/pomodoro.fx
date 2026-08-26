// pomodoro.fx
// A Pomodoro Timer app written in f(x)

let time = import("std:time")
let ui = import("std:topia")

// Configuration
let WORK_MINUTES = 25
let BREAK_MINUTES = 5

// State
var running = false
var end_time = 0
var remaining = WORK_MINUTES * 60
var is_work_mode = true

let start_timer = func() {
    running = true
    end_time = time.now_secs() + remaining
}

let pause_timer = func() {
    running = false
}

let reset_timer = func() {
    running = false
    if is_work_mode {
        remaining = WORK_MINUTES * 60
    } else {
        remaining = BREAK_MINUTES * 60
    }
}

let toggle_mode = func() {
    is_work_mode = !is_work_mode
    reset_timer()
}

// Since f(x) might not have integer division natively yet, we can approximate
// or use subtraction in a loop for integer modulo and division.
let get_minutes_and_seconds = func(total_secs) {
    var secs = total_secs
    var mins = 0
    while secs >= 60 {
        secs = secs - 60
        mins = mins + 1
    }
    return [mins, secs]
}

let render = func() {
    // Update logic
    if running {
        let current = time.now_secs()
        remaining = end_time - current
        if remaining <= 0 {
            remaining = 0
            running = false
            // Auto-switch mode on completion
            is_work_mode = !is_work_mode
            reset_timer()
        }
    }
    
    // Formatting
    let time_parts = get_minutes_and_seconds(remaining)
    let m = time_parts[0]
    let s = time_parts[1]
    
    let s_str = if s < 10 { "0" + s } else { s + "" }
    let m_str = if m < 10 { "0" + m } else { m + "" }
    let display = m_str + ":" + s_str
    
    let mode_text = if is_work_mode { "Work Focus" } else { "Break Time" }
    
    // UI Layout
    ui.Center(ui.VStack([
        ui.Text("Pomodoro", {"size": 32, "bold": true}),
        ui.Text(mode_text, {"size": 20}),
        ui.Text(display, {"size": 64, "bold": true}),
        
        ui.HStack([
            if running {
                ui.Button("Pause", pause_timer)
            } else {
                ui.Button("Start", start_timer)
            },
            ui.Button("Reset", reset_timer)
        ], {"spacing": 20}),
        
        ui.Empty(),
        
        ui.Button("Switch to " + (if is_work_mode { "Break" } else { "Work" }), toggle_mode)
    ], {"spacing": 15}))
}

let app = ui.App("f(x) Pomodoro", 400, 450)
ui.run(app, render)
