let topia = import("topia")
let app = topia.App("Topia Multi-Panel Dashboard", 600, 500)
var active_tab = "Home"
var tab_visits = 0
var notifications = 3
let tab_home = topia.Button("Home Tab", func() {
    active_tab = "Home"
    tab_visits = tab_visits + 1
})
let tab_analytics = topia.Button("Analytics Tab", func() {
    active_tab = "Analytics"
    tab_visits = tab_visits + 1
})
let tab_settings = topia.Button("Settings Tab", func() {
    active_tab = "Settings"
    tab_visits = tab_visits + 1
})
let btn_clear_notif = topia.Button("Clear Alerts", func() {
    notifications = 0
})
let view = func() {
    let header = topia.HStack([topia.Text("System Dashboard"), topia.Text("Active: " + active_tab), topia.Text("Alerts: " + notifications)], 15)
    let nav_bar = topia.HStack([tab_home, tab_analytics, tab_settings, btn_clear_notif], 8)
    var body_content = topia.Empty()
    if active_tab == "Home" {
        body_content = topia.VStack([topia.Text("Welcome to Home View"), topia.Text("Total tab navigation switches: " + tab_visits)], 5)
    } else {
        if active_tab == "Analytics" {
            body_content = topia.VStack([topia.Text("Analytics & Metrics"), topia.Text("System load: Optimal"), topia.Text("Visits: " + tab_visits)], 5)
        } else {
            body_content = topia.VStack([topia.Text("User Settings Panel"), topia.Text("Notification Count: " + notifications)], 5)
        }
    }
    return topia.VStack([header, topia.Text("----------------------------------------"), nav_bar, topia.Text("----------------------------------------"), body_content], 10)
}
app.run(view)
