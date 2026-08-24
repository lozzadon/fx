let ui = import("std:topia")

struct Task {
    name: String,
    completed: Bool,
}

var tasks = [
    Task("Learn f(x)", true),
    Task("Build Topia MVP", false)
]
var new_task_name = ""

let add_task = func() {
    if len(new_task_name) > 0 {
        push(tasks, Task(new_task_name, false))
        new_task_name = ""
    }
}

let clear_tasks = func() {
    var empty = []
    tasks = empty
}

let app_builder = func() {
    var task_nodes = []
    
    var i = 0
    while i < len(tasks) {
        let idx = i
        let task = tasks[idx]
        
        let on_toggle = func(checked) {
            tasks[idx].completed = checked
        }
        
        let label = task.name
        if task.completed {
            push(task_nodes, ui.Checkbox(task.completed, label, on_toggle))
        } else {
            push(task_nodes, ui.Checkbox(task.completed, label, on_toggle))
        }
        i = i + 1
    }
    
    if len(tasks) == 0 {
        push(task_nodes, ui.Text("No tasks yet. Take a break!"))
    }
    
    let on_input_change = func(val) {
        new_task_name = val
    }
    
    ui.VStack([
        ui.Text("f(x) Task Manager", {"size": 24, "bold": true}),
        ui.HStack([
            ui.TextInput(new_task_name, on_input_change),
            ui.Button("Add", add_task)
        ]),
        ui.HStack([
            ui.Button("Clear Tasks", clear_tasks)
        ]),
        ui.VStack(task_nodes)
    ])
}

let app = ui.App("Task Manager", 400, 500)
ui.run(app, app_builder)
