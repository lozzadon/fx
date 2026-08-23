let topia = import("topia")
let app = topia.App("Topia Counter Demo", 400, 300)
var count = 0
let btn_dec = topia.Button("-", func() {
    count = count - 1
})
let btn_inc = topia.Button("+", func() {
    count = count + 1
})
let btn_reset = topia.Button("Reset", func() {
    count = 0
})
let view = func() {
    topia.VStack([topia.Text("Topia Counter Demo"), topia.Text("Count: " + count), topia.HStack([btn_dec, btn_inc, btn_reset])])
}
app.run(view)
