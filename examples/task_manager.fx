let ui = import("std:topia")

var tasks = []

let add_task = func() {
    let new_task = "Task " + (len(tasks) + 1)
    push(tasks, new_task)
}

let clear_tasks = func() {
    var empty = []
    tasks = empty
}

let app_builder = func() {
    var task_nodes = []
    
    var i = 0
    while i < len(tasks) {
        let task_name = tasks[i]
        push(task_nodes, ui.Text("• " + task_name))
        i = i + 1
    }
    
    if len(tasks) == 0 {
        push(task_nodes, ui.Text("No tasks yet. Take a break!"))
    }
    
    ui.VStack([
        ui.Text("f(x) Task Manager"),
        ui.HStack([
            ui.Button("Add Task", add_task),
            ui.Button("Clear Tasks", clear_tasks)
        ]),
        ui.VStack(task_nodes)
    ])
}

let app = ui.App("Task Manager", 400, 500)
ui.run(app, app_builder)
