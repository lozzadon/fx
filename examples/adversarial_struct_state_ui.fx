let topia = import("topia")
let app = topia.App("Topia Struct State App", 400, 350)
struct UserProfile {
    name: String,
    level: Int,
    points: Int,
    is_vip: Bool,
}
var user = UserProfile("Alice", 1, 100, false)
let btn_level_up = topia.Button("Level Up", func() {
    user.level = user.level + 1
    user.points = user.points + 50
    if user.level >= 5 {
        user.is_vip = true
    }
})
let btn_spend = topia.Button("Spend 30 Pts", func() {
    if user.points >= 30 {
        user.points = user.points - 30
    }
})
let btn_reset_user = topia.Button("Reset Profile", func() {
    user = UserProfile("Alice", 1, 100, false)
})
let view = func() {
    topia.VStack([topia.Text("Player: " + user.name), topia.Text("Level: " + user.level), topia.Text("Points: " + user.points), topia.Text("VIP Status: " + user.is_vip), topia.HStack([btn_level_up, btn_spend, btn_reset_user], 8)], 10)
}
app.run(view)
