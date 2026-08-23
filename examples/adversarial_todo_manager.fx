let topia = import("topia")
let app = topia.App("Topia Todo Manager", 450, 400)
var todos = ["Learn Topia", "Build f(x) UI"]
var last_action = "Initial state"
let btn_add = topia.Button("Add Item", func() {
    push(todos, "Task " + len(todos) + 1)
    last_action = "Added Task " + len(todos)
})
let btn_pop = topia.Button("Remove Last", func() {
    if len(todos) > 0 {
        let removed = pop(todos)
        last_action = "Removed item: " + removed
    } else {
        last_action = "No items left to remove"
    }
})
let btn_clear = topia.Button("Clear All", func() {
    todos = []
    last_action = "Cleared all tasks"
})
let view = func() {
    var item_nodes = []
    for item in todos {
        push(item_nodes, topia.Text("- " + item))
    }
    if len(todos) == 0 {
        push(item_nodes, topia.Text("(No tasks available)"))
    }
    let items_stack = topia.VStack(item_nodes, 4)
    let action_bar = topia.HStack([btn_add, btn_pop, btn_clear], 8)
    return topia.VStack([topia.Text("=== Dynamic Todo List ==="), topia.Text("Total Tasks: " + len(todos)), topia.Text("Status: " + last_action), topia.Text("-------------------------"), items_stack, topia.Text("-------------------------"), action_bar], 6)
}
app.run(view)
