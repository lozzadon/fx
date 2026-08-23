print("================================================================================")
print("  f(x) Advanced Features Showcase (Proposals 3, 5, 6)")
print("================================================================================")
print("")
print("--- 1. Container Element Mutation & Shared Reference Semantics (Proposal 3) ---")
var scores = [10, 20, 30, 40]
print("Initial array: " + scores)
scores[1] = 95
scores[3] = scores[3] + 5
print("After scores[1] = 95 and scores[3] += 5: " + scores)
var grid = [["-", "-", "-"], ["-", "-", "-"], ["-", "-", "-"]]
grid[0][0] = "X"
grid[1][1] = "O"
grid[2][2] = "X"
print("Tic-Tac-Toe diagonal grid:")
print("  Row 0: " + grid[0])
print("  Row 1: " + grid[1])
print("  Row 2: " + grid[2])
let swap = func(arr: Array, i: Int, j: Int) {
    let tmp = arr[i]
    arr[i] = arr[j]
    arr[j] = tmp
}
let bubble_sort = func(arr: Array) {
    let n = len(arr)
    var i = 0
    while i < n {
        var j = 0
        while j < n - i - 1 {
            if arr[j] > arr[j + 1] {
                swap(arr, j, j + 1)
            }
            j = j + 1
        }
        i = i + 1
    }
}
var numbers = [64, 34, 25, 12, 22, 11, 90]
print("Unsorted numbers: " + numbers)
bubble_sort(numbers)
print("Sorted in-place by bubble_sort(numbers): " + numbers)
var inventory = {"apples": 10, "bananas": 5, "oranges": 8}
inventory["apples"] = 15
inventory["grapes"] = 20
print("Mutated dictionary: " + inventory)
print("")
print("--- 2. Struct Records, Field Typing & Dot-Notation Access (Proposal 5) ---")
struct Point {
    x: Int,
    y: Int,
}
struct User {
    id: Int,
    name: String,
    role: String,
    active: Bool,
}
var pt1 = Point(10, 20)
var pt2 = Point(4, 12)
print("Point 1: " + pt1)
print("Point 2: " + pt2)
print("pt1.x = " + pt1.x + ", pt1.y = " + pt1.y)
pt1.x = pt1.x + 5
pt1.y = pt1.y * 2
print("After pt1.x += 5 and pt1.y *= 2: " + pt1)
let distance_squared = func(p1: Point, p2: Point) -> Int {
    let dx = p1.x - p2.x
    let dy = p1.y - p2.y
    return dx * dx + dy * dy
}
let dist_sq = distance_squared(pt1, pt2)
print("distance_squared(pt1, pt2) = " + dist_sq)
var user = User(101, "Alice", "Developer", true)
print("User profile: " + user)
user.role = "Lead Architect"
user.name = "Alice Vance"
print("Updated user profile: " + user)
var config = {"theme": "solarized-dark", "auto_save": true, "tab_size": 4}
print("Config theme: " + config.theme)
config.tab_size = 2
config.theme = "monokai"
print("Updated config: " + config)
print("")
print("--- 3. Modular Standard Library Architecture (Proposal 6) ---")
let math = import("std:math")
print("Math Constants:")
print("  PI = " + math.PI)
print("  E  = " + math.E)
let sqrt_val = math.sqrt(144)
let pow_val = math.pow(2, 10)
let abs_val = math.abs(-42)
let min_val = math.min(50, 25)
let max_val = math.max(50, 25)
let round_val = math.round(3.75)
print("Math Calculations:")
print("  sqrt(144.0) = " + sqrt_val)
print("  pow(2, 10)  = " + pow_val)
print("  abs(-42)    = " + abs_val)
print("  min(50, 25) = " + min_val)
print("  max(50, 25) = " + max_val)
print("  round(3.75) = " + round_val)
let fs = import("std:fs")
let test_file = "/tmp/fx_advanced_demo.txt"
let write_content = "f(x) Standard Library File I/O Test Content
Line 2: Fast, Safe, Dynamic."
let write_result = fs.write_file(test_file, write_content)
if write_result.ok {
    print("std:fs write_file successfully written to " + test_file)
    let exists_result = fs.exists(test_file)
    print("std:fs exists(" + test_file + ") = " + exists_result.val)
    let read_result = fs.read_file(test_file)
    if read_result.ok {
        print("std:fs read_file read " + len(read_result.val) + " bytes:")
        print("--- File Contents ---")
        print(read_result.val)
        print("---------------------")
    }
    let remove_result = fs.remove_file(test_file)
    print("std:fs remove_file cleanup ok: " + remove_result.ok)
    fs.write_file_or_throw(test_file, "Throw variant content")
    let direct_content = fs.read_file_or_throw(test_file)
    print("std:fs read_file_or_throw direct content: " + direct_content)
    fs.remove_file(test_file)
    try {
        fs.read_file_or_throw("/nonexistent/file/path.txt")
    } catch err {
        print("std:fs caught expected throw on missing file: " + err)
    }
} else {
    print("std:fs write_file failed with error: " + write_result.err)
}
let json = import("std:json")
let raw_json = "{"project":"f(x)","version":2.0,"features":["mutation","structs","stdlib"],"active":true}"
print("Raw JSON input: " + raw_json)
let parsed = json.parse(raw_json)
print("Parsed JSON object: " + parsed)
print("  project:  " + parsed.project)
print("  version:  " + parsed.version)
print("  features: " + parsed.features)
parsed.features[0] = "shared_mutation"
let serialized = json.stringify(parsed)
print("Re-serialized JSON: " + serialized)
let os = import("std:os")
print("Operating System Info:")
print("  Platform: " + os.platform())
print("  PID:      " + os.getpid())
let path_var = os.get_env("PATH")
if path_var != null {
    print("  PATH length: " + len(path_var) + " characters")
}
let time = import("std:time")
let t_start = time.now_ms()
print("Time at start: " + t_start + " ms (epoch seconds: " + time.now_secs() + ")")
time.sleep_ms(20)
let t_end = time.now_ms()
let elapsed = t_end - t_start
print("Elapsed after sleep_ms(20): " + elapsed + " ms")
print("")
print("================================================================================")
print("  ALL ADVANCED FEATURES (PROPOSALS 3, 5, 6) EXECUTED SUCCESSFULLY!")
print("================================================================================")
