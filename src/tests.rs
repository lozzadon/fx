use std::rc::Rc;
use std::cell::RefCell;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::object::{Environment, Object};
use crate::evaluator::eval_program;
use crate::compiler::Compiler;
use crate::vm::VM;

fn run_ast(input: &str) -> Object {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    assert!(parser.errors.is_empty(), "Parser errors: {:?}", parser.errors);
    let env = Rc::new(RefCell::new(Environment::new()));
    eval_program(program, env)
}

fn run_vm(input: &str) -> Result<Object, String> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    assert!(parser.errors.is_empty(), "Parser errors: {:?}", parser.errors);
    let mut compiler = Compiler::new();
    compiler.compile(&program)?;
    let mut machine = VM::new(compiler.bytecode());
    machine.run()?;
    Ok(machine.last_popped_elem().cloned().unwrap_or(Object::Null))
}

// =============================================================================
// PROPOSAL 3 TESTS: Container Mutation & Shared References
// =============================================================================

#[test]
fn test_proposal3_array_in_place_mutation() {
    let input = "
        var arr = [10, 20, 30]
        arr[1] = 99
        arr[1]
    ";
    assert_eq!(run_ast(input), Object::Integer(99));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(99));
}

#[test]
fn test_proposal3_matrix_nested_mutation() {
    let input = "
        var matrix = [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]
        matrix[1][1] = 999
        matrix[1][1]
    ";
    assert_eq!(run_ast(input), Object::Integer(999));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(999));
}

#[test]
fn test_proposal3_pass_by_reference_swap() {
    let input = "
        func swap(arr: Array, i: Int, j: Int) {
            let temp = arr[i]
            arr[i] = arr[j]
            arr[j] = temp
        }

        var nums = [100, 200, 300]
        swap(nums, 0, 2)
        nums[0] + nums[2]
    ";
    assert_eq!(run_ast(input), Object::Integer(400));
}

#[test]
fn test_proposal3_dict_mutation() {
    let input = r#"
        var user = {"name": "Alice", "score": 50}
        user["score"] = 95
        user["score"]
    "#;
    assert_eq!(run_ast(input), Object::Integer(95));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(95));
}

#[test]
fn test_proposal3_compound_index_assign() {
    let input = "
        var arr = [10, 20, 30]
        arr[0] += 5
        arr[1] *= 2
        arr[2] -= 10
        arr[0] + arr[1] + arr[2]
    ";
    assert_eq!(run_ast(input), Object::Integer(15 + 40 + 20));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(75));
}

#[test]
fn test_proposal3_array_append_on_boundary() {
    let input = "
        var arr = [1, 2]
        arr[2] = 3
        len(arr)
    ";
    assert_eq!(run_ast(input), Object::Integer(3));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(3));
}

#[test]
fn test_proposal3_cycle_safe_display() {
    let input = "
        var arr = [1]
        arr[0] = arr
        len(arr)
    ";
    let res = run_ast(input);
    assert_eq!(res, Object::Integer(1));
}

// =============================================================================
// PROPOSAL 5 TESTS: Struct Records, Field Typing & Dot-Notation
// =============================================================================

#[test]
fn test_proposal5_struct_declaration_and_instantiation() {
    let input = r#"
        struct Point {
            x: Int,
            y: Int
        }
        var p = Point(10, 20)
        p.x + p.y
    "#;
    assert_eq!(run_ast(input), Object::Integer(30));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(30));
}

#[test]
fn test_proposal5_struct_field_mutation() {
    let input = r#"
        struct User {
            id: Int,
            name: String,
            is_admin: Bool
        }
        var u = User(1, "Alice", false)
        u.name = "Alice Smith"
        u.is_admin = true
        u.name
    "#;
    assert_eq!(run_ast(input), Object::String("Alice Smith".to_string()));
    assert_eq!(run_vm(input).unwrap(), Object::String("Alice Smith".to_string()));
}

#[test]
fn test_proposal5_struct_compound_field_assignment() {
    let input = r#"
        struct Counter {
            count: Int
        }
        var c = Counter(10)
        c.count += 5
        c.count *= 2
        c.count
    "#;
    assert_eq!(run_ast(input), Object::Integer(30));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(30));
}

#[test]
fn test_proposal5_struct_type_contracts() {
    let input = r#"
        struct Point {
            x: Int,
            y: Int
        }
        func add_points(p: Point, q: Point) -> Point {
            return Point(p.x + q.x, p.y + q.y)
        }
        let p1 = Point(1, 2)
        let p2 = Point(3, 4)
        let p3 = add_points(p1, p2)
        p3.x * 10 + p3.y
    "#;
    assert_eq!(run_ast(input), Object::Integer(46));
}

#[test]
fn test_proposal5_struct_constructor_type_error() {
    let input = r#"
        struct Point {
            x: Int,
            y: Int
        }
        Point(1, "invalid")
    "#;
    let res = run_ast(input);
    assert!(matches!(res, Object::Error(_)));
}

#[test]
fn test_proposal5_dict_dot_notation() {
    let input = r#"
        var config = {"theme": "dark", "zoom": 100}
        config.theme = "light"
        config.zoom += 25
        config.theme
    "#;
    assert_eq!(run_ast(input), Object::String("light".to_string()));
    assert_eq!(run_vm(input).unwrap(), Object::String("light".to_string()));
}

// =============================================================================
// PROPOSAL 6 TESTS: Modular Standard Library
// =============================================================================

#[test]
fn test_proposal6_std_math() {
    let input = r#"
        let math = import("std:math")
        let a = math.abs(-42)
        let b = math.sqrt(25.0)
        let c = math.pow(2, 3)
        let d = math.floor(3.7)
        let e = math.ceil(3.2)
        let f = math.min(10, 5)
        let g = math.max(10, 5)
        a + b + c + d + e + f + g
    "#;
    assert_eq!(run_ast(input), Object::Float(42.0 + 5.0 + 8.0 + 3.0 + 4.0 + 5.0 + 10.0));
}

#[test]
fn test_proposal6_std_math_constants() {
    let input = r#"
        let math = import("std:math")
        math.PI > 3.14 && math.E > 2.71
    "#;
    assert_eq!(run_ast(input), Object::Boolean(true));
}

#[test]
fn test_proposal6_std_fs_structured_result() {
    let tmp_path = "/tmp/fx_test_fs_prop6.txt";
    let input = format!(r#"
        let fs = import("std:fs")
        let write_res = fs.write_file("{}", "Hello f(x) standard library!")
        if write_res.ok {{
            let read_res = fs.read_file("{}")
            if read_res.ok {{
                read_res.val
            }} else {{
                "read_failed"
            }}
        }} else {{
            "write_failed"
        }}
    "#, tmp_path, tmp_path);
    assert_eq!(run_ast(&input), Object::String("Hello f(x) standard library!".to_string()));

    // Cleanup
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_proposal6_std_json() {
    let input = r#"
        let json = import("std:json")
        let raw = "\{\"name\":\"f(x)\",\"version\":1,\"tags\":[\"fast\",\"dynamic\"]\}"
        let data = json.parse(raw)
        let encoded = json.stringify(data)
        let roundtrip = json.parse(encoded)
        roundtrip.name
    "#;
    assert_eq!(run_ast(input), Object::String("f(x)".to_string()));
}

#[test]
fn test_proposal6_std_os() {
    let input = r#"
        let os = import("std:os")
        let pid = os.getpid()
        let plat = os.platform()
        pid > 0 && len(plat) > 0
    "#;
    assert_eq!(run_ast(input), Object::Boolean(true));
}

#[test]
fn test_proposal6_std_time() {
    let input = r#"
        let time = import("std:time")
        let t1 = time.now_ms()
        let t2 = time.now_secs()
        t1 > 0 && t2 > 0
    "#;
    assert_eq!(run_ast(input), Object::Boolean(true));
}

// =============================================================================
// ADVERSARIAL STRESS & EDGE CASE TESTS
// =============================================================================

#[test]
fn test_circular_reference_multi_struct_equality() {
    let input = r#"
        struct Node {
            id: Int,
            next: Any
        }
        var n1 = Node(1, null)
        var n2 = Node(2, null)
        n1.next = n2
        n2.next = n1

        var m1 = Node(1, null)
        var m2 = Node(2, null)
        m1.next = m2
        m2.next = m1

        n1 == m1 && n1 == n1
    "#;
    assert_eq!(run_ast(input), Object::Boolean(true));
}

#[test]
fn test_circular_reference_json_stringify_safe() {
    let input = r#"
        let json = import("std:json")
        var arr = [1, 2]
        arr[0] = arr
        let str = json.stringify(arr)
        str
    "#;
    let res = run_ast(input);
    if let Object::String(s) = res {
        assert_eq!(s, "[null,2]");
    } else {
        panic!("expected string, got {:?}", res);
    }
}

#[test]
fn test_json_surrogate_pairs_decoding() {
    let input = r#"
        let json = import("std:json")
        let parsed = json.parse("\{ \"emoji\": \"\uD83D\uDE00 \uD83D\uDCA9\" \}")
        parsed.emoji
    "#;
    assert_eq!(run_ast(input), Object::String("😀 💩".to_string()));
}

#[test]
fn test_fs_and_os_sandboxing_capabilities() {
    // 1. Test allow_fs = false
    crate::stdlib::set_config(crate::stdlib::FxConfig {
        allow_fs: false,
        allow_os: true,
        fs_root: None,
        max_file_size: 1024,
    });
    let input_no_fs = r#"
        let fs = import("std:fs")
        fs
    "#;
    let res_no_fs = run_ast(input_no_fs);
    assert!(matches!(res_no_fs, Object::Error(_)));

    // 2. Test allow_os = false
    crate::stdlib::set_config(crate::stdlib::FxConfig {
        allow_fs: true,
        allow_os: false,
        fs_root: None,
        max_file_size: 1024,
    });
    let input_no_os = r#"
        let os = import("std:os")
        os
    "#;
    let res_no_os = run_ast(input_no_os);
    assert!(matches!(res_no_os, Object::Error(_)));

    // 3. Test fs_root jail
    let sandbox_dir = std::env::temp_dir().join("fx_sandbox_test");
    let _ = std::fs::create_dir_all(&sandbox_dir);
    crate::stdlib::set_config(crate::stdlib::FxConfig {
        allow_fs: true,
        allow_os: true,
        fs_root: Some(sandbox_dir.clone()),
        max_file_size: 1024,
    });
    let input_traversal = r#"
        let fs = import("std:fs")
        let res = fs.read_file("/etc/passwd")
        res.ok
    "#;
    assert_eq!(run_ast(input_traversal), Object::Boolean(false));

    // Reset default config
    crate::stdlib::set_config(crate::stdlib::FxConfig::default());
    let _ = std::fs::remove_dir_all(&sandbox_dir);
}

#[test]
fn test_formatter_struct_and_assignment_roundtrip() {
    use crate::formatter::format_program;

    let input = "struct Point {\n    x: Int,\n    y: Int,\n}\nvar p = Point(10, 20)\np.x += 5\n";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    assert_eq!(parser.errors.len(), 0);

    let formatted = format_program(&program);
    let lexer2 = Lexer::new(&formatted);
    let mut parser2 = Parser::new(lexer2);
    let program2 = parser2.parse_program();
    assert_eq!(parser2.errors.len(), 0);
    assert_eq!(program.statements.len(), program2.statements.len());
}

#[test]
fn test_nested_struct_in_array_and_dict() {
    let input = r#"
        struct Point {
            x: Int,
            y: Int
        }
        var pts = [Point(1, 2), Point(3, 4)]
        pts[0].x = 10
        pts[1].y += 6
        pts[0].x + pts[1].y
    "#;
    assert_eq!(run_ast(input), Object::Integer(20));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(20));
}

#[test]
fn test_nested_dict_in_array_mutation() {
    let input = r#"
        var data = [{"user": "Alice", "score": 10}, {"user": "Bob", "score": 20}]
        data[0].score += 5
        data[1].score = 30
        data[0].score + data[1].score
    "#;
    assert_eq!(run_ast(input), Object::Integer(45));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(45));
}

#[test]
fn test_math_error_guards() {
    let input_sqrt_neg = r#"
        let math = import("std:math")
        math.sqrt(-4.0)
    "#;
    assert!(matches!(run_ast(input_sqrt_neg), Object::Error(_)));

    let input_log_zero = r#"
        let math = import("std:math")
        math.log(0.0)
    "#;
    assert!(matches!(run_ast(input_log_zero), Object::Error(_)));
}

#[test]
fn test_fs_missing_file_result_handling() {
    let input = r#"
        let fs = import("std:fs")
        let read_res = fs.read_file("/non/existent/fx_test_file_missing.txt")
        let exists_res = fs.exists("/non/existent/fx_test_file_missing.txt")
        let remove_res = fs.remove_file("/non/existent/fx_test_file_missing.txt")
        read_res.ok == false && exists_res.val == false && remove_res.ok == false
    "#;
    assert_eq!(run_ast(input), Object::Boolean(true));
}

#[test]
fn test_struct_constructor_arity_and_nominal_type_error() {
    let input_too_few = r#"
        struct Point {
            x: Int,
            y: Int
        }
        Point(1)
    "#;
    assert!(matches!(run_ast(input_too_few), Object::Error(_)));

    let input_too_many = r#"
        struct Point {
            x: Int,
            y: Int
        }
        Point(1, 2, 3)
    "#;
    assert!(matches!(run_ast(input_too_many), Object::Error(_)));

    let input_wrong_type = r#"
        struct User {
            name: String,
            age: Int
        }
        User("Alice", "twenty")
    "#;
    assert!(matches!(run_ast(input_wrong_type), Object::Error(_)));
}

#[test]
fn test_array_out_of_bounds_errors() {
    let input_neg = r#"
        var arr = [1, 2, 3]
        arr[-1] = 5
    "#;
    assert!(matches!(run_ast(input_neg), Object::Error(_)));

    let input_past_len = r#"
        var arr = [1, 2, 3]
        arr[5] = 5
    "#;
    assert!(matches!(run_ast(input_past_len), Object::Error(_)));
}

#[test]
fn test_len_across_all_collection_types() {
    let input = r#"
        struct Pair {
            first: Int,
            second: Int
        }
        let str_len = len("hello")
        let arr_len = len([1, 2, 3, 4])
        let dict_len = len({"a": 1, "b": 2, "c": 3})
        let struct_len = len(Pair(10, 20))
        let range_len = len(0..5)
        str_len + arr_len + dict_len + struct_len + range_len
    "#;
    assert_eq!(run_ast(input), Object::Integer(5 + 4 + 3 + 2 + 5));
}

#[test]
fn test_fs_read_write_throw_variants() {
    let tmp_path = "/tmp/fx_test_fs_throw_variants.txt";
    let input_write = format!(r#"
        let fs = import("std:fs")
        fs.write_file_or_throw("{}", "throw variant content")
    "#, tmp_path);
    assert_eq!(run_ast(&input_write), Object::Boolean(true));

    let input_read = format!(r#"
        let fs = import("std:fs")
        fs.read_file_or_throw("{}")
    "#, tmp_path);
    assert_eq!(run_ast(&input_read), Object::String("throw variant content".to_string()));

    let input_try_catch = r#"
        let fs = import("std:fs")
        var caught = "none"
        try {
            fs.read_file_or_throw("/nonexistent/file/for/sure.txt")
        } catch e {
            caught = "caught_error"
        }
        caught
    "#;
    assert_eq!(run_ast(input_try_catch), Object::String("caught_error".to_string()));

    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_fs_nonexistent_ancestor_sandbox_jail() {
    let sandbox_dir = std::env::temp_dir().join("fx_sandbox_ancestor_test");
    let _ = std::fs::create_dir_all(&sandbox_dir);
    crate::stdlib::set_config(crate::stdlib::FxConfig {
        allow_fs: true,
        allow_os: true,
        fs_root: Some(sandbox_dir.clone()),
        max_file_size: 1024,
    });

    let input_escape = r#"
        let fs = import("std:fs")
        let res = fs.write_file("../../nonexistent_ancestor_dir/evil.txt", "payload")
        res.ok
    "#;
    assert_eq!(run_ast(input_escape), Object::Boolean(false));

    crate::stdlib::set_config(crate::stdlib::FxConfig::default());
    let _ = std::fs::remove_dir_all(&sandbox_dir);
}

#[test]
fn test_os_direct_apply_capability_gate() {
    crate::stdlib::set_config(crate::stdlib::FxConfig {
        allow_fs: true,
        allow_os: false,
        fs_root: None,
        max_file_size: 1024,
    });

    let direct_call_res = crate::evaluator::apply_builtin("std:os:getpid", vec![]);
    assert!(matches!(direct_call_res, Object::Error(_)));

    crate::stdlib::set_config(crate::stdlib::FxConfig::default());
}

#[test]
fn test_vm_hash_literal_duplicate_keys_order() {
    let input = r#"
        let d = {"a": 1, "a": 2, "a": 3}
        d.a
    "#;
    assert_eq!(run_ast(input), Object::Integer(3));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(3));
}

#[test]
fn test_deeply_nested_struct_mutation() {
    let input = r#"
        struct Meta {
            tag: String,
            score: Int
        }
        struct Item {
            id: Int,
            meta: Meta
        }
        var store = {
            "items": [Item(1, Meta("alpha", 10)), Item(2, Meta("beta", 20))]
        }
        store.items[1].meta.tag = "beta_updated"
        store.items[1].meta.score += 5
        store.items[1].meta.tag + ": " + store.items[1].meta.score
    "#;
    assert_eq!(run_ast(input), Object::String("beta_updated: 25".to_string()));
    assert_eq!(run_vm(input).unwrap(), Object::String("beta_updated: 25".to_string()));
}

#[test]
fn test_struct_bracket_index_and_mutation() {
    let input = r#"
        struct Point {
            x: Int,
            y: Int
        }
        var p = Point(10, 20)
        p["x"] = 99
        p["x"] + p["y"]
    "#;
    assert_eq!(run_ast(input), Object::Integer(119));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(119));
}

#[test]
fn test_fs_all_throw_variants() {
    let tmp_dir = "/tmp/fx_test_fs_all_throws";
    let tmp_file = "/tmp/fx_test_fs_all_throws/test.txt";
    let input = format!(r#"
        let fs = import("std:fs")
        fs.create_dir_or_throw("{}")
        fs.write_file_or_throw("{}", "hello")
        fs.append_file_or_throw("{}", " world")
        let content = fs.read_file_or_throw("{}")
        fs.remove_file_or_throw("{}")
        content
    "#, tmp_dir, tmp_file, tmp_file, tmp_file, tmp_file);
    assert_eq!(run_ast(&input), Object::String("hello world".to_string()));

    let _ = std::fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_matrix_compound_element_mutation() {
    let input = r#"
        var matrix = [
            [1, 2],
            [3, 4]
        ]
        matrix[0][1] += 10
        matrix[1][0] *= 5
        matrix[0][1] + matrix[1][0]
    "#;
    assert_eq!(run_ast(input), Object::Integer(12 + 15));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(27));
}

#[test]
fn test_nested_dict_compound_dot_mutation() {
    let input = r#"
        var config = {
            "server": {
                "port": 8080,
                "active": true
            }
        }
        config.server.port += 20
        config.server.port
    "#;
    assert_eq!(run_ast(input), Object::Integer(8100));
    assert_eq!(run_vm(input).unwrap(), Object::Integer(8100));
}

#[test]
fn test_math_pow_nan_guard() {
    let input = r#"
        let math = import("std:math")
        math.pow(-2.0, 0.5)
    "#;
    assert!(matches!(run_ast(input), Object::Error(_)));
}

#[test]
fn test_modulo_by_zero_guards() {
    let input_int = "10 % 0";
    assert!(matches!(run_ast(input_int), Object::Error(_)));
    assert!(run_vm(input_int).is_err());

    let input_float = "10.5 % 0.0";
    assert!(matches!(run_ast(input_float), Object::Error(_)));
    assert!(run_vm(input_float).is_err());
}
