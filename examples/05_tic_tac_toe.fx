// 05_tic_tac_toe.fx
// A full Tic-Tac-Toe game using nested layouts and complex game logic.

let ui = import("std:topia")

var board = [" ", " ", " ", " ", " ", " ", " ", " ", " "]
var current_turn = "X"
var winner = ""
var status_text = "Current Turn: X"

var update_status = func() {
    if winner != "" {
        if winner == "Tie" {
            status_text = "It's a Tie!"
        } else {
            status_text = winner + " Wins!"
        }
    } else {
        status_text = "Current Turn: " + current_turn
    }
}

var check_winner = func() {
    var win_patterns = [
        [0, 1, 2], [3, 4, 5], [6, 7, 8],
        [0, 3, 6], [1, 4, 7], [2, 5, 8],
        [0, 4, 8], [2, 4, 6]
    ]
    
    for p in win_patterns {
        var a = p[0]
        var b = p[1]
        var c = p[2]
        if board[a] != " " {
            if board[a] == board[b] {
                if board[b] == board[c] {
                    winner = board[a]
                    break
                }
            }
        }
    }
    
    if winner == "" {
        var is_tie = true
        for cell in board {
            if cell == " " {
                is_tie = false
                break
            }
        }
        if is_tie {
            winner = "Tie"
        }
    }
    
    update_status()
}

var reset_game = func() {
    for i in 0..9 {
        board[i] = " "
    }
    current_turn = "X"
    winner = ""
    update_status()
}

var make_cell = func(index) {
    ui.Button(board[index], func() {
        if winner == "" {
            if board[index] == " " {
                board[index] = current_turn
                if current_turn == "X" {
                    current_turn = "O"
                } else {
                    current_turn = "X"
                }
                check_winner()
            }
        }
    })
}

var app_builder = func() {
    var rows = []
    for row_idx in 0..3 {
        var cols = []
        for col_idx in 0..3 {
            var idx = row_idx * 3 + col_idx
            push(cols, make_cell(idx))
        }
        push(rows, ui.HStack(cols))
    }
    
    ui.VStack([
        ui.Text("Tic-Tac-Toe", {"size": 32, "bold": true}),
        ui.Text(status_text, {"size": 20}),
        ui.VStack(rows),
        ui.Button("Reset Game", reset_game)
    ])
}

let app = ui.App("Tic-Tac-Toe", 300, 400)
ui.run(app, app_builder)
