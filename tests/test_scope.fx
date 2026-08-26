var x = 10
for i in 0..3 {
    let x = i
    print(x)
}
print(x) // should be 10

let y = 5
if y == 5 {
    let y = 100
    print(y)
}
print(y) // should be 5
