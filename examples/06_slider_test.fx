let ui = import("std:topia")

var count = 0
var scale = 1.0

let render = func() {
    var items = []
    
    // Add 20 items to demonstrate scrolling
    var i = 0
    while i < 20 {
        let label = "Scrollable Item #" + i
        push(items, ui.Text(label, {"size": 14 * scale}))
        i = i + 1
    }

    ui.VStack([
        ui.Text("Topia Slider & ScrollArea Test", {"size": 24 * scale, "bold": true}),
        
        ui.HStack([
            ui.Text("UI Scale: ", {"size": 16 * scale}),
            ui.Slider(scale, 0.5, 3.0, func(new_val) {
                scale = new_val
            })
        ]),
        
        ui.Text("Count: " + count, {"size": 18 * scale}),
        ui.Button("Increment", func() {
            count = count + 1
        }),
        
        ui.Text("Scrollable Content Below:", {"size": 16 * scale, "bold": true}),
        
        ui.ScrollArea(items)
    ], 10.0) // 10.0 spacing
}

let app = ui.App("Slider & Scroll Test", 500, 600)
ui.run(app, render)
