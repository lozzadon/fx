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

// =============================================================================
// TOPIA DECLARATIVE UI FRAMEWORK TESTS (Milestone 2)
// =============================================================================

#[test]
fn test_topia_import_and_module_structure() {
    let input = r#"
        let topia = import("topia")
        let res = [
            topia._module,
            topia.App != null,
            topia.Text != null,
            topia.Button != null,
            topia.VStack != null,
            topia.HStack != null,
            topia.run != null
        ]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("topia".to_string()),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_import_prefixes_idempotency() {
    let input = r#"
        let t1 = import("topia")
        let t2 = import("std:topia")
        let t3 = import("std/topia")
        let res = [t1._module, t2._module, t3._module]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("topia".to_string()),
        Object::String("topia".to_string()),
        Object::String("topia".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_app_constructor() {
    let input = r#"
        let topia = import("topia")
        let app = topia.App("Topia Counter", 400, 300)
        let res = [app._type, app.title, app.width, app.height]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("App".to_string()),
        Object::String("Topia Counter".to_string()),
        Object::Float(400.0),
        Object::Float(300.0),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_text_constructor() {
    let input = r#"
        let topia = import("topia")
        let txt = topia.Text("Hello Topia")
        let res = [txt._type, txt.text]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Text".to_string()),
        Object::String("Hello Topia".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_button_constructor_and_direct_callback() {
    let input = r#"
        let topia = import("topia")
        var count = 0
        let btn = topia.Button("Increment", func() {
            count += 1
            return count
        })
        let res = btn.on_click()
        let out = [btn._type, btn.label, res, count]
        out
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Button".to_string()),
        Object::String("Increment".to_string()),
        Object::Integer(1),
        Object::Integer(1),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_layout_stacks_hierarchy() {
    let input = r#"
        let topia = import("topia")
        let v = topia.VStack([
            topia.Text("Title"),
            topia.HStack([
                topia.Button("-", func() {}),
                topia.Text("0"),
                topia.Button("+", func() {})
            ])
        ])
        let res = [v._type, len(v.children), v.children[1]._type, len(v.children[1].children)]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("VStack".to_string()),
        Object::Integer(2),
        Object::String("HStack".to_string()),
        Object::Integer(3),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_callback_state_mutation_counter() {
    let input = r#"
        let topia = import("topia")
        var count = 10
        var step = 5
        let btn_inc = topia.Button("+", func() { count += step })
        let btn_dec = topia.Button("-", func() { count -= step })
        let btn_reset = topia.Button("Reset", func() { count = 0 })

        btn_inc.on_click()
        btn_inc.on_click()
        let val1 = count
        btn_dec.on_click()
        let val2 = count
        btn_reset.on_click()
        let val3 = count

        let res = [val1, val2, val3]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(20),
        Object::Integer(15),
        Object::Integer(0),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_callback_complex_state_mutations() {
    let input = r#"
        let topia = import("topia")
        var state = {
            "user": "Alice",
            "active": false,
            "tags": ["initial"]
        }
        let toggle_btn = topia.Button("Toggle", func() {
            state.active = !state.active
            push(state.tags, "updated")
        })
        toggle_btn.on_click()
        let res = [state.user, state.active, len(state.tags)]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Alice".to_string()),
        Object::Boolean(true),
        Object::Integer(2),
    ])));
    assert_eq!(run_ast(input), expected);
}

#[test]
fn test_topia_object_to_node_conversion_and_inspection() {
    let input = r#"
        let topia = import("topia")
        topia.VStack([
            topia.Text("Heading"),
            topia.Button("Action", func() { return 100 })
        ])
    "#;
    let eval_obj = run_ast(input);
    let root_node = crate::stdlib::topia::object_to_node(&eval_obj);

    assert_eq!(root_node.child_count(), 2);
    let children = root_node.children();
    assert_eq!(children[0].as_text(), Some("Heading"));
    assert_eq!(children[1].as_button_label(), Some("Action"));
}

#[test]
fn test_topia_headless_reactivity_and_view_re_evaluation() {
    let script = r#"
        let topia = import("topia")
        var count = 0
        let btn = topia.Button("+", func() { count += 1 })
        let view_builder = func() {
            return topia.VStack([
                topia.Text("Count: " + count),
                btn
            ])
        }
        let res = [btn, view_builder]
        res
    "#;
    let eval_obj = run_ast(script);
    let arr = match eval_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };
    let btn_obj = arr[0].clone();
    let vb_obj = arr[1].clone();

    // 1. Initial Frame Render
    let f1_obj = crate::evaluator::apply_function(vb_obj.clone(), vec![]);
    let f1_node = crate::stdlib::topia::object_to_node(&f1_obj);
    assert_eq!(f1_node.children()[0].as_text(), Some("Count: 0"));

    // 2. Fire Button Click (via Node or direct callback)
    let mut btn_node = crate::stdlib::topia::object_to_node(&btn_obj);
    assert!(btn_node.fire_click());

    // 3. Second Frame Render (Reactivity Verification)
    let f2_obj = crate::evaluator::apply_function(vb_obj.clone(), vec![]);
    let f2_node = crate::stdlib::topia::object_to_node(&f2_obj);
    assert_eq!(f2_node.children()[0].as_text(), Some("Count: 1"));

    // 4. Multiple rapid-fire clicks
    assert!(btn_node.fire_click());
    assert!(btn_node.fire_click());
    assert!(btn_node.fire_click());

    // 5. Third Frame Render
    let f3_obj = crate::evaluator::apply_function(vb_obj, vec![]);
    let f3_node = crate::stdlib::topia::object_to_node(&f3_obj);
    assert_eq!(f3_node.children()[0].as_text(), Some("Count: 4"));
}

#[test]
fn test_topia_error_guards_and_type_safety() {
    let input_bad_app = "let t = import(\"topia\")\nt.App(\"Title\", \"not_number\", 500)";
    assert!(matches!(run_ast(input_bad_app), Object::Error(_)));

    let input_bad_btn = "let t = import(\"topia\")\nt.Button(\"Click\", \"not_a_func\")";
    assert!(matches!(run_ast(input_bad_btn), Object::Error(_)));

    let input_bad_vstack = "let t = import(\"topia\")\nt.VStack(\"not_an_array\")";
    assert!(matches!(run_ast(input_bad_vstack), Object::Error(_)));
}

// =============================================================================
// ADVERSARIAL CHALLENGE TESTS (M2 Challenger 2)
// =============================================================================

#[test]
fn test_topia_adversarial_constructor_type_mismatches() {
    // 1. App Constructor Mismatches
    let bad_app_cases = [
        "let t = import(\"topia\")\nt.App()",
        "let t = import(\"topia\")\nt.App(\"Title\")",
        "let t = import(\"topia\")\nt.App(\"Title\", 800)",
        "let t = import(\"topia\")\nt.App(12345, 800, 600)",
        "let t = import(\"topia\")\nt.App([\"title\"], 800, 600)",
        "let t = import(\"topia\")\nt.App({\"t\": 1}, 800, 600)",
        "let t = import(\"topia\")\nt.App(true, 800, 600)",
        "let t = import(\"topia\")\nt.App(null, 800, 600)",
        "let t = import(\"topia\")\nt.App(\"Title\", \"invalid_width\", 600)",
        "let t = import(\"topia\")\nt.App(\"Title\", [800], 600)",
        "let t = import(\"topia\")\nt.App(\"Title\", null, 600)",
        "let t = import(\"topia\")\nt.App(\"Title\", 800, \"invalid_height\")",
        "let t = import(\"topia\")\nt.App(\"Title\", 800, {\"h\": 600})",
        "let t = import(\"topia\")\nt.App(\"Title\", 800, null)",
    ];
    for code in bad_app_cases {
        let res = run_ast(code);
        assert!(matches!(res, Object::Error(_)), "Expected Error for App constructor mismatch: {}\nGot: {:?}", code, res);
    }

    // 2. Button Constructor Mismatches
    let bad_btn_cases = [
        "let t = import(\"topia\")\nt.Button()",
        "let t = import(\"topia\")\nt.Button(\"Only Label\")",
        "let t = import(\"topia\")\nt.Button(\"Label\", func() {}, \"extra_arg\")",
        "let t = import(\"topia\")\nt.Button(999, func() {})",
        "let t = import(\"topia\")\nt.Button([\"label\"], func() {})",
        "let t = import(\"topia\")\nt.Button(null, func() {})",
        "let t = import(\"topia\")\nt.Button(true, func() {})",
        "let t = import(\"topia\")\nt.Button(\"Label\", \"string_not_func\")",
        "let t = import(\"topia\")\nt.Button(\"Label\", 12345)",
        "let t = import(\"topia\")\nt.Button(\"Label\", true)",
        "let t = import(\"topia\")\nt.Button(\"Label\", [1, 2, 3])",
        "let t = import(\"topia\")\nt.Button(\"Label\", {\"key\": \"value\"})",
        "let t = import(\"topia\")\nt.Button(\"Label\", null)",
    ];
    for code in bad_btn_cases {
        let res = run_ast(code);
        assert!(matches!(res, Object::Error(_)), "Expected Error for Button constructor mismatch: {}\nGot: {:?}", code, res);
    }

    // 3. VStack & HStack Constructor Mismatches
    let bad_stack_cases = [
        "let t = import(\"topia\")\nt.VStack()",
        "let t = import(\"topia\")\nt.VStack(\"not_an_array\")",
        "let t = import(\"topia\")\nt.VStack(12345)",
        "let t = import(\"topia\")\nt.VStack(true)",
        "let t = import(\"topia\")\nt.VStack({\"children\": []})",
        "let t = import(\"topia\")\nt.VStack(null)",
        "let t = import(\"topia\")\nt.HStack()",
        "let t = import(\"topia\")\nt.HStack(\"not_an_array\")",
        "let t = import(\"topia\")\nt.HStack(12345)",
        "let t = import(\"topia\")\nt.HStack(true)",
        "let t = import(\"topia\")\nt.HStack({\"children\": []})",
        "let t = import(\"topia\")\nt.HStack(null)",
    ];
    for code in bad_stack_cases {
        let res = run_ast(code);
        assert!(matches!(res, Object::Error(_)), "Expected Error for Stack constructor mismatch: {}\nGot: {:?}", code, res);
    }

    // 4. Text Constructor Arity Mismatches
    let bad_text_cases = [
        "let t = import(\"topia\")\nt.Text()",
        "let t = import(\"topia\")\nt.Text(\"arg1\", \"arg2\")",
        "let t = import(\"topia\")\nt.Text(\"arg1\", 123, 456)",
    ];
    for code in bad_text_cases {
        let res = run_ast(code);
        assert!(matches!(res, Object::Error(_)), "Expected Error for Text constructor arity mismatch: {}\nGot: {:?}", code, res);
    }

    // 5. run Builtin Arity Mismatches
    let bad_run_cases = [
        "let t = import(\"topia\")\nt.run()",
        "let t = import(\"topia\")\nt.run(t.App(\"T\", 100, 100), func() {}, \"extra\")",
    ];
    for code in bad_run_cases {
        let res = run_ast(code);
        assert!(matches!(res, Object::Error(_)), "Expected Error for run arity mismatch: {}\nGot: {:?}", code, res);
    }
}

#[test]
fn test_topia_adversarial_empty_arrays_and_containers() {
    let script = r#"
        let topia = import("topia")
        let empty_v = topia.VStack([])
        let empty_h = topia.HStack([])
        let empty_leaf = topia.Empty()
        let nested_empty = topia.VStack([
            topia.HStack([]),
            topia.Empty(),
            topia.VStack([
                topia.HStack([]),
                topia.Empty()
            ])
        ])
        let res = [
            empty_v._type,
            len(empty_v.children),
            empty_h._type,
            len(empty_h.children),
            empty_leaf._type,
            nested_empty._type,
            len(nested_empty.children)
        ]
        res
    "#;
    let eval_res = run_ast(script);
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("VStack".to_string()),
        Object::Integer(0),
        Object::String("HStack".to_string()),
        Object::Integer(0),
        Object::String("Empty".to_string()),
        Object::String("VStack".to_string()),
        Object::Integer(3),
    ])));
    assert_eq!(eval_res, expected);

    // Convert to Topia Node tree and inspect structure
    let root_node = crate::stdlib::topia::object_to_node(&eval_res);
    // eval_res is an array, so object_to_node wraps it in a VStack
    assert_eq!(root_node.child_count(), 7);

    // Test direct node conversion of nested empty structures
    let empty_tree_script = r#"
        let topia = import("topia")
        topia.VStack([
            topia.HStack([]),
            topia.Empty(),
            topia.VStack([
                topia.HStack([]),
                topia.Empty()
            ])
        ])
    "#;
    let empty_tree_obj = run_ast(empty_tree_script);
    let native_empty_tree = crate::stdlib::topia::object_to_node(&empty_tree_obj);
    assert_eq!(native_empty_tree.child_count(), 3);
    assert_eq!(native_empty_tree.children()[0].child_count(), 0);
    assert!(native_empty_tree.children()[1].is_empty());
    assert_eq!(native_empty_tree.children()[2].child_count(), 2);
    assert_eq!(native_empty_tree.children()[2].children()[0].child_count(), 0);
    assert!(native_empty_tree.children()[2].children()[1].is_empty());

    // Bare objects conversion
    let bare_empty_arr = Object::Array(Rc::new(RefCell::new(vec![])));
    let bare_arr_node = crate::stdlib::topia::object_to_node(&bare_empty_arr);
    assert_eq!(bare_arr_node.child_count(), 0);

    let bare_null = Object::Null;
    let bare_null_node = crate::stdlib::topia::object_to_node(&bare_null);
    assert!(bare_null_node.is_empty());

    let bare_empty_hash = Object::Hash(Rc::new(RefCell::new(std::collections::HashMap::new())));
    let bare_hash_node = crate::stdlib::topia::object_to_node(&bare_empty_hash);
    assert!(bare_hash_node.is_empty());
}

#[test]
fn test_topia_adversarial_unicode_and_special_strings() {
    let script = r#"
        let topia = import("topia")
        let app = topia.App("Topia 界面框架 🚀 日本語・한국어・العربية", 1024, 768)
        let t_cjk = topia.Text("こんにちは世界！你好，世界！안녕하세요!")
        let t_arabic = topia.Text("مرحبا بالعالم - تجربة واجهة المستخدم")
        let t_emoji = topia.Text("🔥🎉👍🚀👨‍👩‍👧‍👦 100% Declarative UI 🌟")
        let t_special = topia.Text("Line 1\nLine 2\tTabbed\r\nLine 3 \\ \"Quotes\"")
        let t_empty = topia.Text("")
        let btn_unicode = topia.Button("✨ 点击计数 / Click +1 🎯", func() { return "ok" })
        let btn_empty = topia.Button("", func() { return "empty_label" })

        let stack = topia.VStack([
            t_cjk,
            t_arabic,
            t_emoji,
            t_special,
            t_empty,
            btn_unicode,
            btn_empty
        ])
        let res = [
            app.title,
            t_cjk.text,
            t_arabic.text,
            t_emoji.text,
            t_special.text,
            t_empty.text,
            btn_unicode.label,
            btn_empty.label,
            stack
        ]
        res
    "#;
    let eval_res = run_ast(script);
    let arr = match &eval_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };
    assert_eq!(arr[0], Object::String("Topia 界面框架 🚀 日本語・한국어・العربية".to_string()));
    assert_eq!(arr[1], Object::String("こんにちは世界！你好，世界！안녕하세요!".to_string()));
    assert_eq!(arr[2], Object::String("مرحبا بالعالم - تجربة واجهة المستخدم".to_string()));
    assert_eq!(arr[3], Object::String("🔥🎉👍🚀👨‍👩‍👧‍👦 100% Declarative UI 🌟".to_string()));
    assert_eq!(arr[4], Object::String("Line 1\nLine 2\tTabbed\r\nLine 3 \\ \"Quotes\"".to_string()));
    assert_eq!(arr[5], Object::String("".to_string()));
    assert_eq!(arr[6], Object::String("✨ 点击计数 / Click +1 🎯".to_string()));
    assert_eq!(arr[7], Object::String("".to_string()));

    // Verify native Node tree conversion preserves strings identically
    let stack_obj = &arr[8];
    let native_node = crate::stdlib::topia::object_to_node(stack_obj);
    assert_eq!(native_node.child_count(), 7);
    let children = native_node.children();
    assert_eq!(children[0].as_text(), Some("こんにちは世界！你好，世界！안녕하세요!"));
    assert_eq!(children[1].as_text(), Some("مرحبا بالعالم - تجربة واجهة المستخدم"));
    assert_eq!(children[2].as_text(), Some("🔥🎉👍🚀👨‍👩‍👧‍👦 100% Declarative UI 🌟"));
    assert_eq!(children[3].as_text(), Some("Line 1\nLine 2\tTabbed\r\nLine 3 \\ \"Quotes\""));
    assert_eq!(children[4].as_text(), Some(""));
    assert_eq!(children[5].as_button_label(), Some("✨ 点击计数 / Click +1 🎯"));
    assert_eq!(children[6].as_button_label(), Some(""));

    // Ultra-long string (10,000 chars) test
    let long_str = "A".repeat(10_000);
    let long_str_obj = Object::String(long_str.clone());
    let long_text_node = crate::stdlib::topia::object_to_node(&long_str_obj);
    assert_eq!(long_text_node.as_text(), Some(long_str.as_str()));
}

#[test]
fn test_topia_adversarial_multi_variable_counter_simulation() {
    let script = r#"
        let topia = import("topia")
        var count_a = 0
        var count_b = 100
        var step = 5
        var multiplier = 2
        var active = false
        var history = []
        var stats = {
            "total_clicks": 0,
            "last_action": "none",
            "derived_sum": 100
        }

        let btn_inc_a = topia.Button("Inc A", func() {
            count_a += step
            stats.total_clicks += 1
            stats.last_action = "inc_a"
            stats.derived_sum = count_a + count_b
            push(history, "A:" + count_a)
        })

        let btn_dec_b = topia.Button("Dec B", func() {
            count_b -= step * multiplier
            stats.total_clicks += 1
            stats.last_action = "dec_b"
            stats.derived_sum = count_a + count_b
            push(history, "B:" + count_b)
        })

        let btn_toggle = topia.Button("Toggle Active", func() {
            active = !active
            stats.total_clicks += 1
            stats.last_action = "toggle"
        })

        let btn_batch_loop = topia.Button("Batch Step", func() {
            for i in 0..4 {
                count_a += 1
                count_b -= 1
            }
            stats.total_clicks += 1
            stats.last_action = "batch"
            stats.derived_sum = count_a + count_b
            push(history, "BATCH")
        })

        let btn_reset = topia.Button("Reset All", func() {
            count_a = 0
            count_b = 100
            active = false
            stats.total_clicks += 1
            stats.last_action = "reset"
            stats.derived_sum = 100
            push(history, "RESET")
        })

        let get_state = func() {
            return [
                count_a,
                count_b,
                active,
                stats.total_clicks,
                stats.last_action,
                stats.derived_sum,
                len(history)
            ]
        }

        let buttons = [btn_inc_a, btn_dec_b, btn_toggle, btn_batch_loop, btn_reset, get_state]
        buttons
    "#;

    let eval_res = run_ast(script);
    let btn_arr = match eval_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array of buttons"),
    };

    let mut inc_a_node = crate::stdlib::topia::object_to_node(&btn_arr[0]);
    let mut dec_b_node = crate::stdlib::topia::object_to_node(&btn_arr[1]);
    let mut toggle_node = crate::stdlib::topia::object_to_node(&btn_arr[2]);
    let mut batch_node = crate::stdlib::topia::object_to_node(&btn_arr[3]);
    let mut reset_node = crate::stdlib::topia::object_to_node(&btn_arr[4]);
    let get_state_fn = btn_arr[5].clone();

    // 1. Initial State Check
    let s0 = crate::evaluator::apply_function(get_state_fn.clone(), vec![]);
    assert_eq!(s0, Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(0),
        Object::Integer(100),
        Object::Boolean(false),
        Object::Integer(0),
        Object::String("none".to_string()),
        Object::Integer(100),
        Object::Integer(0),
    ]))));

    // 2. Click Inc A twice (+5, +5)
    assert!(inc_a_node.fire_click());
    assert!(inc_a_node.fire_click());

    let s1 = crate::evaluator::apply_function(get_state_fn.clone(), vec![]);
    assert_eq!(s1, Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(10),
        Object::Integer(100),
        Object::Boolean(false),
        Object::Integer(2),
        Object::String("inc_a".to_string()),
        Object::Integer(110),
        Object::Integer(2),
    ]))));

    // 3. Click Dec B (100 - (5*2) = 90)
    assert!(dec_b_node.fire_click());

    let s2 = crate::evaluator::apply_function(get_state_fn.clone(), vec![]);
    assert_eq!(s2, Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(10),
        Object::Integer(90),
        Object::Boolean(false),
        Object::Integer(3),
        Object::String("dec_b".to_string()),
        Object::Integer(100),
        Object::Integer(3),
    ]))));

    // 4. Toggle Active flag
    assert!(toggle_node.fire_click());

    let s3 = crate::evaluator::apply_function(get_state_fn.clone(), vec![]);
    assert_eq!(s3, Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(10),
        Object::Integer(90),
        Object::Boolean(true),
        Object::Integer(4),
        Object::String("toggle".to_string()),
        Object::Integer(100),
        Object::Integer(3),
    ]))));

    // 5. Batch step (+4 to A, -4 to B)
    assert!(batch_node.fire_click());

    let s4 = crate::evaluator::apply_function(get_state_fn.clone(), vec![]);
    assert_eq!(s4, Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(14),
        Object::Integer(86),
        Object::Boolean(true),
        Object::Integer(5),
        Object::String("batch".to_string()),
        Object::Integer(100),
        Object::Integer(4),
    ]))));

    // 6. Reset All
    assert!(reset_node.fire_click());

    let s5 = crate::evaluator::apply_function(get_state_fn, vec![]);
    assert_eq!(s5, Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(0),
        Object::Integer(100),
        Object::Boolean(false),
        Object::Integer(6),
        Object::String("reset".to_string()),
        Object::Integer(100),
        Object::Integer(5),
    ]))));
}

#[test]
fn test_topia_adversarial_rapid_view_builder_re_evaluations() {
    let script = r#"
        let topia = import("topia")
        var count = 0
        let btn_inc = topia.Button("Inc", func() { count += 1 })
        let btn_dec = topia.Button("Dec", func() { count -= 1 })

        let view_builder = func() {
            if count % 2 == 0 {
                return topia.VStack([
                    topia.Text("Mode: EVEN (" + count + ")"),
                    topia.HStack([btn_inc, btn_dec])
                ])
            } else {
                return topia.VStack([
                    topia.Text("Mode: ODD (" + count + ")"),
                    topia.Text("Status: Special Odd Frame"),
                    topia.HStack([btn_inc, btn_dec]),
                    topia.Empty()
                ])
            }
        }

        let pack = [btn_inc, btn_dec, view_builder]
        pack
    "#;

    let eval_res = run_ast(script);
    let pack = match eval_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };

    let mut inc_node = crate::stdlib::topia::object_to_node(&pack[0]);
    let mut dec_node = crate::stdlib::topia::object_to_node(&pack[1]);
    let vb_func = pack[2].clone();

    // Rapid simulation of 100 frame evaluations with interleaved click events
    for frame in 0..100 {
        let v_obj = crate::evaluator::apply_function(vb_func.clone(), vec![]);
        let v_node = crate::stdlib::topia::object_to_node(&v_obj);

        if frame % 2 == 0 {
            // Even frame
            assert_eq!(v_node.child_count(), 2);
            assert_eq!(v_node.children()[0].as_text(), Some(format!("Mode: EVEN ({})", frame).as_str()));
        } else {
            // Odd frame
            assert_eq!(v_node.child_count(), 4);
            assert_eq!(v_node.children()[0].as_text(), Some(format!("Mode: ODD ({})", frame).as_str()));
            assert_eq!(v_node.children()[1].as_text(), Some("Status: Special Odd Frame"));
            assert!(v_node.children()[3].is_empty());
        }

        // Advance count by 1 for next frame
        assert!(inc_node.fire_click());
    }

    // Now count is 100. Rapidly decrement 50 times in a tight loop and re-evaluate view builder
    for _ in 0..50 {
        assert!(dec_node.fire_click());
    }

    // After 50 decrements from 100, count is 50 (Even)
    let v_50_obj = crate::evaluator::apply_function(vb_func.clone(), vec![]);
    let v_50_node = crate::stdlib::topia::object_to_node(&v_50_obj);
    assert_eq!(v_50_node.child_count(), 2);
    assert_eq!(v_50_node.children()[0].as_text(), Some("Mode: EVEN (50)"));

    // Stress: 500 consecutive view builder evaluations without state change
    for _ in 0..500 {
        let stress_obj = crate::evaluator::apply_function(vb_func.clone(), vec![]);
        let stress_node = crate::stdlib::topia::object_to_node(&stress_obj);
        assert_eq!(stress_node.child_count(), 2);
        assert_eq!(stress_node.children()[0].as_text(), Some("Mode: EVEN (50)"));
    }
}

#[test]
fn test_topia_adversarial_heterogeneous_children_and_loose_hashes() {
    let script = r#"
        let topia = import("topia")
        let loose_text = {"type": "Text", "text": "Loose Text Object"}
        let loose_btn = {"type": "Button", "label": "Loose Btn", "callback": func() { return 42 }}
        let loose_vstack = {"children": [loose_text, loose_btn]}
        let raw_literals_stack = topia.VStack([
            "Raw String Child",
            12345,
            67.89,
            true,
            null,
            loose_vstack
        ])
        raw_literals_stack
    "#;

    let eval_res = run_ast(script);
    let native_node = crate::stdlib::topia::object_to_node(&eval_res);

    assert_eq!(native_node.child_count(), 6);
    let children = native_node.children();
    assert_eq!(children[0].as_text(), Some("Raw String Child"));
    assert_eq!(children[1].as_text(), Some("12345"));
    assert_eq!(children[2].as_text(), Some("67.89"));
    assert_eq!(children[3].as_text(), Some("true"));
    assert!(children[4].is_empty());
    assert_eq!(children[5].child_count(), 2);
    assert_eq!(children[5].children()[0].as_text(), Some("Loose Text Object"));
    assert_eq!(children[5].children()[1].as_button_label(), Some("Loose Btn"));

    let loose_btn_obj = match &eval_res {
        Object::Hash(rc) => {
            let map = rc.borrow();
            if let Some(Object::Array(arr)) = map.get(&crate::object::HashKey::String("children".to_string())) {
                let vstack_child = &arr.borrow()[5];
                if let Object::Hash(v_map) = vstack_child {
                    if let Some(Object::Array(v_arr)) = v_map.borrow().get(&crate::object::HashKey::String("children".to_string())) {
                        v_arr.borrow()[1].clone()
                    } else {
                        Object::Null
                    }
                } else {
                    Object::Null
                }
            } else {
                Object::Null
            }
        }
        _ => Object::Null,
    };
    let mut loose_btn_node = crate::stdlib::topia::object_to_node(&loose_btn_obj);
    assert!(loose_btn_node.fire_click());
}

// =============================================================================
// MILESTONE 3: VM FUNCTIONS & VM TOPIA PARITY TEST SUITE
// =============================================================================

// -----------------------------------------------------------------------------
// Suite 0: VM Function & Closure Parity
// -----------------------------------------------------------------------------

#[test]
fn test_vm_function_literal_and_call() {
    let input = r#"
        func add(a, b) {
            return a + b
        }
        add(15, 27)
    "#;
    assert_eq!(run_ast(input), Object::Integer(42));
    assert_eq!(run_vm(input).expect("VM failed on function call"), Object::Integer(42));
}

#[test]
fn test_vm_higher_order_functions_and_closures() {
    let input = r#"
        func make_adder(x) {
            return func(y) {
                return x + y
            }
        }
        let add5 = make_adder(5)
        add5(10)
    "#;
    assert_eq!(run_ast(input), Object::Integer(15));
    assert_eq!(run_vm(input).expect("VM failed on closure"), Object::Integer(15));
}

#[test]
fn test_vm_function_with_type_annotations() {
    let input = r#"
        func multiply(x: Int, y: Int) -> Int {
            return x * y
        }
        multiply(6, 7)
    "#;
    assert_eq!(run_ast(input), Object::Integer(42));
    assert_eq!(run_vm(input).expect("VM failed on typed function"), Object::Integer(42));
}

// -----------------------------------------------------------------------------
// Suite 1: VM Topia Module Import & Resolution Parity
// -----------------------------------------------------------------------------

#[test]
fn test_vm_topia_import_and_module_structure() {
    let input = r#"
        let topia = import("topia")
        let res = [
            topia._module,
            topia.App != null,
            topia.Text != null,
            topia.Button != null,
            topia.VStack != null,
            topia.HStack != null,
            topia.Empty != null,
            topia.run != null
        ]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("topia".to_string()),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
    ])));
    assert_eq!(run_ast(input), expected, "AST failed on topia module structure");
    assert_eq!(run_vm(input).expect("VM failed on topia module structure"), expected);
}

#[test]
fn test_vm_topia_import_prefixes_idempotency() {
    let input = r#"
        let t1 = import("topia")
        let t2 = import("std:topia")
        let t3 = import("std/topia")
        let res = [t1._module, t2._module, t3._module]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("topia".to_string()),
        Object::String("topia".to_string()),
        Object::String("topia".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on import prefixes"), expected);
}

#[test]
fn test_vm_topia_import_in_block_scope() {
    let input = r#"
        var result = ""
        if true {
            let topia = import("topia")
            let txt = topia.Text("Scoped Import")
            result = txt.text
        }
        result
    "#;
    let expected = Object::String("Scoped Import".to_string());
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on scoped import"), expected);
}

#[test]
fn test_vm_topia_import_index_and_dot_access_parity() {
    let input = r#"
        let topia = import("topia")
        let t_dot = topia.Text("Dot Access")
        let t_idx = topia["Text"]("Index Access")
        let res = [t_dot.text, t_idx.text]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Dot Access".to_string()),
        Object::String("Index Access".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on dot/index access"), expected);
}

// -----------------------------------------------------------------------------
// Suite 2: VM Topia Declarative UI Constructors Parity
// -----------------------------------------------------------------------------

#[test]
fn test_vm_topia_app_constructor() {
    let input = r#"
        let topia = import("topia")
        let app1 = topia.App("Main Window", 800, 600)
        let app2 = topia.App("Float Window", 640.5, 480.5)
        let res = [
            app1._type, app1.title, app1.width, app1.height, app1.resizable,
            app2._type, app2.title, app2.width, app2.height, app2.resizable
        ]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("App".to_string()),
        Object::String("Main Window".to_string()),
        Object::Float(800.0),
        Object::Float(600.0),
        Object::Boolean(true),
        Object::String("App".to_string()),
        Object::String("Float Window".to_string()),
        Object::Float(640.5),
        Object::Float(480.5),
        Object::Boolean(true),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on App constructor"), expected);
}

#[test]
fn test_vm_topia_text_constructor_variations() {
    let input = r#"
        let topia = import("topia")
        let count = 42
        let t_plain = topia.Text("Hello VM")
        let t_concat = topia.Text("Items: " + count)
        let t_empty = topia.Text("")
        let t_special = topia.Text("Line1\nLine2\tTabbed")
        let t_unicode = topia.Text("🚀 Topia 界面 🌟")
        let res = [t_plain.text, t_concat.text, t_empty.text, t_special.text, t_unicode.text]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Hello VM".to_string()),
        Object::String("Items: 42".to_string()),
        Object::String("".to_string()),
        Object::String("Line1\nLine2\tTabbed".to_string()),
        Object::String("🚀 Topia 界面 🌟".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on Text constructor"), expected);
}

#[test]
fn test_vm_topia_button_constructor_and_structure() {
    let input = r#"
        let topia = import("topia")
        let btn = topia.Button("Submit", func() { return 100 })
        let res = [btn._type, btn.label, btn.on_click != null, btn.callback != null]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Button".to_string()),
        Object::String("Submit".to_string()),
        Object::Boolean(true),
        Object::Boolean(true),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on Button constructor"), expected);
}

#[test]
fn test_vm_topia_vstack_constructor_and_spacing() {
    let input = r#"
        let topia = import("topia")
        let v_empty = topia.VStack([])
        let v_items = topia.VStack([topia.Text("A"), topia.Text("B")], 15.5)
        let res = [
            v_empty._type, len(v_empty.children),
            v_items._type, len(v_items.children), v_items.spacing, v_items.children[0].text
        ]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("VStack".to_string()),
        Object::Integer(0),
        Object::String("VStack".to_string()),
        Object::Integer(2),
        Object::Float(15.5),
        Object::String("A".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on VStack constructor"), expected);
}

#[test]
fn test_vm_topia_hstack_constructor_and_spacing() {
    let input = r#"
        let topia = import("topia")
        let h_empty = topia.HStack([])
        let h_items = topia.HStack([topia.Text("X"), topia.Text("Y")], 10.0)
        let res = [
            h_empty._type, len(h_empty.children),
            h_items._type, len(h_items.children), h_items.spacing, h_items.children[1].text
        ]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("HStack".to_string()),
        Object::Integer(0),
        Object::String("HStack".to_string()),
        Object::Integer(2),
        Object::Float(10.0),
        Object::String("Y".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on HStack constructor"), expected);
}

#[test]
fn test_vm_topia_empty_constructor() {
    let input = r#"
        let topia = import("topia")
        let emp = topia.Empty()
        emp._type
    "#;
    let expected = Object::String("Empty".to_string());
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on Empty constructor"), expected);
}

// -----------------------------------------------------------------------------
// Suite 3: VM Button Callback Execution & State Mutation Parity
// -----------------------------------------------------------------------------

#[test]
fn test_vm_topia_button_direct_callback_invocation() {
    let input = r#"
        let topia = import("topia")
        var value = 50
        let btn = topia.Button("Add", func() {
            value = value + 25
            return value
        })
        let ret = btn.on_click()
        let res = [ret, value]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(75),
        Object::Integer(75),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on direct callback invocation"), expected);
}

#[test]
fn test_vm_topia_callback_scalar_state_mutations() {
    let input = r#"
        let topia = import("topia")
        var count = 0
        var active = false
        var log = "Init"

        let btn_inc = topia.Button("Inc", func() { count += 5 })
        let btn_toggle = topia.Button("Toggle", func() { active = !active })
        let btn_log = topia.Button("Log", func() { log = log + " -> Clicked" })

        btn_inc.on_click()
        btn_inc.on_click()
        btn_toggle.on_click()
        btn_log.on_click()

        let res = [count, active, log]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(10),
        Object::Boolean(true),
        Object::String("Init -> Clicked".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on scalar state mutations"), expected);
}

#[test]
fn test_vm_topia_callback_container_state_mutations() {
    let input = r#"
        let topia = import("topia")
        var arr = [10, 20]
        var dict = {"status": "pending", "hits": 0}

        let btn = topia.Button("Mutate", func() {
            push(arr, 30)
            arr[0] = 99
            dict.status = "done"
            dict.hits += 1
        })

        btn.on_click()

        let res = [arr[0], arr[2], len(arr), dict.status, dict.hits]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(99),
        Object::Integer(30),
        Object::Integer(3),
        Object::String("done".to_string()),
        Object::Integer(1),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on container state mutations"), expected);
}

#[test]
fn test_vm_topia_callback_struct_state_mutation() {
    let input = r#"
        let topia = import("topia")
        struct State {
            count: Int,
            label: String
        }
        var s = State(0, "initial")
        let btn = topia.Button("Update", func() {
            s.count += 10
            s.label = "updated"
        })
        btn.on_click()
        let res = [s.count, s.label]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(10),
        Object::String("updated".to_string()),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on struct state mutation"), expected);
}

#[test]
fn test_vm_topia_multi_button_shared_state_machine() {
    let input = r#"
        let topia = import("topia")
        var count = 0
        var step = 1
        var history = []

        let btn_step_5 = topia.Button("Step 5", func() { step = 5 })
        let btn_inc = topia.Button("+", func() {
            count += step
            push(history, "+" + step)
        })
        let btn_dec = topia.Button("-", func() {
            count -= step
            push(history, "-" + step)
        })
        let btn_reset = topia.Button("Reset", func() {
            count = 0
            step = 1
            push(history, "R")
        })

        btn_inc.on_click()     // count = 1
        btn_step_5.on_click()  // step = 5
        btn_inc.on_click()     // count = 6
        btn_inc.on_click()     // count = 11
        btn_dec.on_click()     // count = 6
        btn_reset.on_click()   // count = 0, step = 1

        let res = [count, step, len(history)]
        res
    "#;
    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(0),
        Object::Integer(1),
        Object::Integer(5),
    ])));
    assert_eq!(run_ast(input), expected);
    assert_eq!(run_vm(input).expect("VM failed on multi-button state machine"), expected);
}

// -----------------------------------------------------------------------------
// Suite 4: Native Topia Node Conversion & Reactivity Parity
// -----------------------------------------------------------------------------

#[test]
fn test_vm_topia_object_to_node_conversion_parity() {
    let input = r#"
        let topia = import("topia")
        topia.VStack([
            topia.Text("Header"),
            topia.HStack([
                topia.Button("Dec", func() {}),
                topia.Text("0"),
                topia.Button("Inc", func() {})
            ])
        ])
    "#;
    let ast_res = run_ast(input);
    let vm_res = run_vm(input).expect("VM failed on UI tree evaluation");

    let ast_node = crate::stdlib::topia::object_to_node(&ast_res);
    let vm_node = crate::stdlib::topia::object_to_node(&vm_res);

    assert_eq!(ast_node.child_count(), vm_node.child_count());
    assert_eq!(ast_node.child_count(), 2);
    assert_eq!(vm_node.children()[0].as_text(), Some("Header"));
    assert_eq!(vm_node.children()[1].child_count(), 3);
    assert_eq!(vm_node.children()[1].children()[0].as_button_label(), Some("Dec"));
    assert_eq!(vm_node.children()[1].children()[1].as_text(), Some("0"));
    assert_eq!(vm_node.children()[1].children()[2].as_button_label(), Some("Inc"));
}

#[test]
fn test_vm_topia_node_fire_click_reactivity_parity() {
    let script = r#"
        let topia = import("topia")
        var count = 0
        let btn = topia.Button("+", func() { count += 1 })
        let view_builder = func() {
            return topia.VStack([
                topia.Text("Count: " + count),
                btn
            ])
        }
        let pack = [btn, view_builder]
        pack
    "#;

    let vm_obj = run_vm(script).expect("VM failed to run reactive script");
    let pack = match vm_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack from VM"),
    };
    let btn_obj = pack[0].clone();
    let vb_obj = pack[1].clone();

    // 1. Initial Frame Render
    let f1_obj = crate::evaluator::apply_function(vb_obj.clone(), vec![]);
    let f1_node = crate::stdlib::topia::object_to_node(&f1_obj);
    assert_eq!(f1_node.children()[0].as_text(), Some("Count: 0"));

    // 2. Fire Button Click via Native Node
    let mut btn_node = crate::stdlib::topia::object_to_node(&btn_obj);
    assert!(btn_node.fire_click(), "Button click failed to fire");

    // 3. Second Frame Render
    let f2_obj = crate::evaluator::apply_function(vb_obj.clone(), vec![]);
    let f2_node = crate::stdlib::topia::object_to_node(&f2_obj);
    assert_eq!(f2_node.children()[0].as_text(), Some("Count: 1"));

    // 4. Multiple rapid clicks
    assert!(btn_node.fire_click());
    assert!(btn_node.fire_click());
    assert!(btn_node.fire_click());

    // 5. Final Frame Render
    let f3_obj = crate::evaluator::apply_function(vb_obj, vec![]);
    let f3_node = crate::stdlib::topia::object_to_node(&f3_obj);
    assert_eq!(f3_node.children()[0].as_text(), Some("Count: 4"));
}

#[test]
fn test_vm_topia_deep_nested_layout_hierarchy() {
    let script = r#"
        let topia = import("topia")
        let root = topia.VStack([
            topia.HStack([
                topia.VStack([
                    topia.HStack([
                        topia.Text("Deep Leaf"),
                        topia.Button("Deep Action", func() { return 999 })
                    ])
                ])
            ])
        ])
        root
    "#;
    let vm_obj = run_vm(script).expect("VM failed on deep layout");
    let native_node = crate::stdlib::topia::object_to_node(&vm_obj);

    assert_eq!(native_node.child_count(), 1);
    let level1 = &native_node.children()[0];
    assert_eq!(level1.child_count(), 1);
    let level2 = &level1.children()[0];
    assert_eq!(level2.child_count(), 1);
    let level3 = &level2.children()[0];
    assert_eq!(level3.child_count(), 2);
    assert_eq!(level3.children()[0].as_text(), Some("Deep Leaf"));
    assert_eq!(level3.children()[1].as_button_label(), Some("Deep Action"));
}

#[test]
fn test_vm_topia_dynamic_loop_generated_children() {
    let script = r#"
        let topia = import("topia")
        var items = []
        for i in 0..5 {
            push(items, topia.Text("Item " + i))
        }
        topia.VStack(items)
    "#;
    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on loop-generated children");

    let ast_node = crate::stdlib::topia::object_to_node(&ast_res);
    let vm_node = crate::stdlib::topia::object_to_node(&vm_res);

    assert_eq!(ast_node.child_count(), 5);
    assert_eq!(vm_node.child_count(), 5);
    for i in 0..5 {
        assert_eq!(vm_node.children()[i].as_text(), Some(format!("Item {}", i).as_str()));
    }
}

// -----------------------------------------------------------------------------
// Suite 5: Real-World Applications & Counter Demo Parity
// -----------------------------------------------------------------------------

#[test]
fn test_vm_topia_e2e_counter_demo_parity() {
    let script = r#"
        let topia = import("topia")
        let app = topia.App("Topia Counter", 400, 300)
        var count = 0
        var step = 1

        let btn_dec = topia.Button("-", func() { count -= step })
        let btn_inc = topia.Button("+", func() { count += step })
        let btn_step_1 = topia.Button("Step 1", func() { step = 1 })
        let btn_step_5 = topia.Button("Step 5", func() { step = 5 })
        let btn_reset = topia.Button("Reset", func() {
            count = 0
            step = 1
        })

        let render_view = func() {
            return topia.VStack([
                topia.Text("Topia Counter App"),
                topia.Text("Count: " + count + " (Step: " + step + ")"),
                topia.HStack([btn_dec, btn_inc]),
                topia.HStack([btn_step_1, btn_step_5, btn_reset])
            ])
        }

        let pack = [btn_dec, btn_inc, btn_step_1, btn_step_5, btn_reset, render_view]
        pack
    "#;

    let vm_obj = run_vm(script).expect("VM failed on counter demo");
    let pack = match vm_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };

    let mut dec_node = crate::stdlib::topia::object_to_node(&pack[0]);
    let mut inc_node = crate::stdlib::topia::object_to_node(&pack[1]);
    let mut step1_node = crate::stdlib::topia::object_to_node(&pack[2]);
    let mut step5_node = crate::stdlib::topia::object_to_node(&pack[3]);
    let mut reset_node = crate::stdlib::topia::object_to_node(&pack[4]);
    let render_fn = pack[5].clone();

    // 1. Initial State
    let v0 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n0 = crate::stdlib::topia::object_to_node(&v0);
    assert_eq!(n0.children()[1].as_text(), Some("Count: 0 (Step: 1)"));

    // 2. Click Inc (+1)
    assert!(inc_node.fire_click());
    let v1 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n1 = crate::stdlib::topia::object_to_node(&v1);
    assert_eq!(n1.children()[1].as_text(), Some("Count: 1 (Step: 1)"));

    // 3. Click Step 5
    assert!(step5_node.fire_click());

    // 4. Click Inc twice (+5, +5 -> 11)
    assert!(inc_node.fire_click());
    assert!(inc_node.fire_click());
    let v2 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n2 = crate::stdlib::topia::object_to_node(&v2);
    assert_eq!(n2.children()[1].as_text(), Some("Count: 11 (Step: 5)"));

    // 5. Click Dec (-5 -> 6)
    assert!(dec_node.fire_click());
    let v3 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n3 = crate::stdlib::topia::object_to_node(&v3);
    assert_eq!(n3.children()[1].as_text(), Some("Count: 6 (Step: 5)"));

    // 6. Click Reset (0, step 1)
    assert!(reset_node.fire_click());
    let v4 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n4 = crate::stdlib::topia::object_to_node(&v4);
    assert_eq!(n4.children()[1].as_text(), Some("Count: 0 (Step: 1)"));

    // 7. Click Step 1 and Inc (+1 -> 1)
    assert!(step1_node.fire_click());
    assert!(inc_node.fire_click());
    let v5 = crate::evaluator::apply_function(render_fn, vec![]);
    let n5 = crate::stdlib::topia::object_to_node(&v5);
    assert_eq!(n5.children()[1].as_text(), Some("Count: 1 (Step: 1)"));
}

#[test]
fn test_vm_topia_e2e_tab_navigation_dashboard() {
    let script = r#"
        let topia = import("topia")
        var active_tab = "Home"
        var visits = 0

        let btn_home = topia.Button("Home", func() {
            active_tab = "Home"
            visits += 1
        })
        let btn_settings = topia.Button("Settings", func() {
            active_tab = "Settings"
            visits += 1
        })

        let render_view = func() {
            let nav = topia.HStack([btn_home, btn_settings])
            var body = topia.Empty()
            if active_tab == "Home" {
                body = topia.Text("Home Tab (Visits: " + visits + ")")
            } else {
                body = topia.Text("Settings Tab (Visits: " + visits + ")")
            }
            return topia.VStack([nav, body])
        }

        let pack = [btn_home, btn_settings, render_view]
        pack
    "#;

    let vm_obj = run_vm(script).expect("VM failed on tab navigation");
    let pack = match vm_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };

    let mut home_node = crate::stdlib::topia::object_to_node(&pack[0]);
    let mut settings_node = crate::stdlib::topia::object_to_node(&pack[1]);
    let render_fn = pack[2].clone();

    // 1. Start at Home
    let v0 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n0 = crate::stdlib::topia::object_to_node(&v0);
    assert_eq!(n0.children()[1].as_text(), Some("Home Tab (Visits: 0)"));

    // 2. Switch to Settings
    assert!(settings_node.fire_click());
    let v1 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n1 = crate::stdlib::topia::object_to_node(&v1);
    assert_eq!(n1.children()[1].as_text(), Some("Settings Tab (Visits: 1)"));

    // 3. Switch back to Home
    assert!(home_node.fire_click());
    let v2 = crate::evaluator::apply_function(render_fn, vec![]);
    let n2 = crate::stdlib::topia::object_to_node(&v2);
    assert_eq!(n2.children()[1].as_text(), Some("Home Tab (Visits: 2)"));
}

#[test]
fn test_vm_topia_e2e_todo_list_manager() {
    let script = r#"
        let topia = import("topia")
        var items = ["Item 1", "Item 2"]
        let btn_add = topia.Button("Add", func() {
            push(items, "Item " + (len(items) + 1))
        })
        let render_view = func() {
            var rows = []
            for item in items {
                push(rows, topia.Text(item))
            }
            push(rows, btn_add)
            return topia.VStack(rows)
        }
        let pack = [btn_add, render_view]
        pack
    "#;

    let vm_obj = run_vm(script).expect("VM failed on todo list");
    let pack = match vm_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };
    let mut add_node = crate::stdlib::topia::object_to_node(&pack[0]);
    let render_fn = pack[1].clone();

    // 1. Initial items: 2 text nodes + 1 button
    let v0 = crate::evaluator::apply_function(render_fn.clone(), vec![]);
    let n0 = crate::stdlib::topia::object_to_node(&v0);
    assert_eq!(n0.child_count(), 3);
    assert_eq!(n0.children()[0].as_text(), Some("Item 1"));
    assert_eq!(n0.children()[1].as_text(), Some("Item 2"));

    // 2. Add item
    assert!(add_node.fire_click());
    let v1 = crate::evaluator::apply_function(render_fn, vec![]);
    let n1 = crate::stdlib::topia::object_to_node(&v1);
    assert_eq!(n1.child_count(), 4);
    assert_eq!(n1.children()[2].as_text(), Some("Item 3"));
}

// -----------------------------------------------------------------------------
// Suite 6: Error Handling, Arity & Type Safety Parity in VM Mode
// -----------------------------------------------------------------------------

#[test]
fn test_vm_topia_constructor_type_errors() {
    let bad_app = "let t = import(\"topia\")\nt.App(\"Title\", \"bad_width\", 600)";
    assert!(matches!(run_ast(bad_app), Object::Error(_)));
    assert!(run_vm(bad_app).is_err() || matches!(run_vm(bad_app).unwrap(), Object::Error(_)));

    let bad_btn = "let t = import(\"topia\")\nt.Button(\"Click\", \"not_func\")";
    assert!(matches!(run_ast(bad_btn), Object::Error(_)));
    assert!(run_vm(bad_btn).is_err() || matches!(run_vm(bad_btn).unwrap(), Object::Error(_)));

    let bad_stack = "let t = import(\"topia\")\nt.VStack(12345)";
    assert!(matches!(run_ast(bad_stack), Object::Error(_)));
    assert!(run_vm(bad_stack).is_err() || matches!(run_vm(bad_stack).unwrap(), Object::Error(_)));
}

#[test]
fn test_vm_topia_constructor_arity_errors() {
    let arity_app = "let t = import(\"topia\")\nt.App(\"OnlyTitle\")";
    assert!(matches!(run_ast(arity_app), Object::Error(_)));
    assert!(run_vm(arity_app).is_err() || matches!(run_vm(arity_app).unwrap(), Object::Error(_)));

    let arity_btn = "let t = import(\"topia\")\nt.Button(\"OnlyLabel\")";
    assert!(matches!(run_ast(arity_btn), Object::Error(_)));
    assert!(run_vm(arity_btn).is_err() || matches!(run_vm(arity_btn).unwrap(), Object::Error(_)));

    let arity_vstack = "let t = import(\"topia\")\nt.VStack()";
    assert!(matches!(run_ast(arity_vstack), Object::Error(_)));
    assert!(run_vm(arity_vstack).is_err() || matches!(run_vm(arity_vstack).unwrap(), Object::Error(_)));
}

#[test]
fn test_vm_topia_adversarial_rapid_callback_stress() {
    let script = r#"
        let topia = import("topia")
        var counter = 0
        let btn = topia.Button("+", func() { counter += 1 })
        for i in 0..500 {
            btn.on_click()
        }
        counter
    "#;
    assert_eq!(run_ast(script), Object::Integer(500));
    assert_eq!(run_vm(script).expect("VM failed on rapid callback loop"), Object::Integer(500));
}

// =============================================================================
// EMPIRICAL CHALLENGER 2: ADVERSARIAL DIFFERENTIAL DUAL-ENGINE PARITY SUITE
// =============================================================================

#[test]
fn test_challenger_differential_recursion_and_math() {
    let script = r#"
        func fib(n) {
            if n <= 1 {
                return n
            }
            return fib(n - 1) + fib(n - 2)
        }

        func fact(n) {
            if n <= 1 {
                return 1
            }
            return n * fact(n - 1)
        }

        func is_even(n) {
            if n == 0 {
                return true
            }
            return is_odd(n - 1)
        }

        func is_odd(n) {
            if n == 0 {
                return false
            }
            return is_even(n - 1)
        }

        let results = [
            fib(10),
            fact(6),
            is_even(12),
            is_odd(13),
            is_even(15)
        ]
        results
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM execution failed on recursion suite");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(55),
        Object::Integer(720),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(false),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_multi_closure_encapsulation() {
    let script = r#"
        func create_account(owner, initial_balance) {
            var balance = initial_balance
            var log = []

            let deposit = func(amount) {
                balance += amount
                push(log, "deposit:" + amount)
                return balance
            }

            let withdraw = func(amount) {
                if amount > balance {
                    push(log, "declined:" + amount)
                    return false
                }
                balance -= amount
                push(log, "withdraw:" + amount)
                return balance
            }

            let get_balance = func() {
                return balance
            }

            let get_log_count = func() {
                return len(log)
            }

            return {
                "owner": owner,
                "deposit": deposit,
                "withdraw": withdraw,
                "balance": get_balance,
                "log_count": get_log_count
            }
        }

        let acc1 = create_account("Alice", 100)
        let acc2 = create_account("Bob", 50)

        acc1.deposit(50)       // 150
        acc1.withdraw(30)      // 120
        acc2.deposit(200)      // 250
        acc2.withdraw(300)     // declined, 250
        acc2.withdraw(50)      // 200

        let res = [
            acc1.balance(),
            acc1.log_count(),
            acc2.balance(),
            acc2.log_count()
        ]
        res
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on multi closure encapsulation");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(120),
        Object::Integer(2),
        Object::Integer(200),
        Object::Integer(3),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_currying_and_composition() {
    let script = r#"
        func curry3(f) {
            return func(a) {
                return func(b) {
                    return func(c) {
                        return f(a, b, c)
                    }
                }
            }
        }

        func poly(a, b, c) {
            return (a * 100) + (b * 10) + c
        }

        let curried_poly = curry3(poly)
        let with_1 = curried_poly(7)
        let with_2 = with_1(4)
        let res1 = with_2(2)

        func compose(f, g) {
            return func(x) {
                return f(g(x))
            }
        }

        func double(n) { return n * 2 }
        func add_three(n) { return n + 3 }

        let double_then_add = compose(add_three, double)
        let add_then_double = compose(double, add_three)

        let res2 = double_then_add(5)
        let res3 = add_then_double(5)

        let out = [res1, res2, res3]
        out
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on currying and composition");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(742),
        Object::Integer(13),
        Object::Integer(16),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_hof_builtins_with_topia_ui() {
    let script = r#"
        let topia = import("topia")

        let numbers = [1, 2, 3, 4, 5, 6, 7, 8]
        let evens = filter(numbers, func(x) { return x % 2 == 0 })
        let doubled_evens = map(evens, func(x) { return x * 2 })
        let sum_evens = reduce(doubled_evens, 0, func(acc, x) { return acc + x })

        var click_count = 0
        let buttons = map(doubled_evens, func(num) {
            return topia.Button("Val " + num, func() {
                click_count += num
            })
        })

        let root = topia.VStack([
            topia.Text("Sum: " + sum_evens),
            topia.HStack(buttons)
        ])

        let out = [sum_evens, len(buttons), root]
        out
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on HOF builtins with Topia");

    let ast_arr = match ast_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };
    let vm_arr = match vm_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };

    assert_eq!(ast_arr[0], Object::Integer(40)); // (2+4+6+8)*2 = 40
    assert_eq!(vm_arr[0], Object::Integer(40));
    assert_eq!(ast_arr[1], Object::Integer(4));
    assert_eq!(vm_arr[1], Object::Integer(4));

    let ast_node = crate::stdlib::topia::object_to_node(&ast_arr[2]);
    let vm_node = crate::stdlib::topia::object_to_node(&vm_arr[2]);

    assert_eq!(ast_node.child_count(), 2);
    assert_eq!(vm_node.child_count(), 2);
    assert_eq!(vm_node.children()[0].as_text(), Some("Sum: 40"));
    assert_eq!(vm_node.children()[1].child_count(), 4);
    assert_eq!(vm_node.children()[1].children()[0].as_button_label(), Some("Val 4"));
    assert_eq!(vm_node.children()[1].children()[3].as_button_label(), Some("Val 16"));
}

#[test]
fn test_challenger_differential_nested_struct_reference_mutation_in_closure() {
    let script = r#"
        struct Coord {
            x: Int,
            y: Int
        }

        struct Entity {
            name: String,
            pos: Coord,
            history: Array
        }

        var player = Entity("Hero", Coord(0, 0), [])

        let move_by = func(dx, dy) {
            player.pos.x += dx
            player.pos.y += dy
            push(player.history, player.pos.x + ":" + player.pos.y)
        }

        move_by(5, 10)
        move_by(-2, 3)
        move_by(10, -5)

        let summary = [player.name, player.pos.x, player.pos.y, len(player.history), player.history[2]]
        summary
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on struct reference mutation in closure");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::String("Hero".to_string()),
        Object::Integer(13),
        Object::Integer(8),
        Object::Integer(3),
        Object::String("13:8".to_string()),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_multi_button_reactive_flow_with_state_resets() {
    let script = r#"
        let topia = import("topia")

        var count = 100
        var multiplier = 2
        var is_active = true
        var log = []

        let btn_toggle = topia.Button("Toggle", func() {
            is_active = !is_active
            push(log, "toggle:" + is_active)
        })

        let btn_calc = topia.Button("Compute", func() {
            if is_active {
                count = (count * multiplier) - 10
            } else {
                count += 1
            }
            push(log, "calc:" + count)
        })

        let btn_set_mult = topia.Button("Mult3", func() {
            multiplier = 3
            push(log, "mult:3")
        })

        let view_builder = func() {
            let status = if is_active { "ACTIVE" } else { "PAUSED" }
            return topia.VStack([
                topia.Text("Status: " + status + " | Value: " + count),
                topia.HStack([btn_toggle, btn_calc, btn_set_mult]),
                topia.Text("Logs: " + len(log))
            ])
        }

        let pack = [btn_toggle, btn_calc, btn_set_mult, view_builder]
        pack
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on multi-button reactive script");

    let vm_pack = match vm_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };
    let ast_pack = match ast_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };

    let mut vm_btn_toggle = crate::stdlib::topia::object_to_node(&vm_pack[0]);
    let mut vm_btn_calc = crate::stdlib::topia::object_to_node(&vm_pack[1]);
    let mut vm_btn_mult = crate::stdlib::topia::object_to_node(&vm_pack[2]);
    let vm_vb = vm_pack[3].clone();

    let mut ast_btn_toggle = crate::stdlib::topia::object_to_node(&ast_pack[0]);
    let mut ast_btn_calc = crate::stdlib::topia::object_to_node(&ast_pack[1]);
    let mut ast_btn_mult = crate::stdlib::topia::object_to_node(&ast_pack[2]);
    let ast_vb = ast_pack[3].clone();

    // Frame 0
    let f0_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let f0_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    let n0_vm = crate::stdlib::topia::object_to_node(&f0_vm);
    let n0_ast = crate::stdlib::topia::object_to_node(&f0_ast);
    assert_eq!(n0_vm.children()[0].as_text(), Some("Status: ACTIVE | Value: 100"));
    assert_eq!(n0_ast.children()[0].as_text(), Some("Status: ACTIVE | Value: 100"));

    // Step 1: Compute -> (100 * 2) - 10 = 190
    assert!(vm_btn_calc.fire_click());
    assert!(ast_btn_calc.fire_click());
    let f1_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let f1_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    let n1_vm = crate::stdlib::topia::object_to_node(&f1_vm);
    let n1_ast = crate::stdlib::topia::object_to_node(&f1_ast);
    assert_eq!(n1_vm.children()[0].as_text(), Some("Status: ACTIVE | Value: 190"));
    assert_eq!(n1_ast.children()[0].as_text(), Some("Status: ACTIVE | Value: 190"));

    // Step 2: Set Mult 3
    assert!(vm_btn_mult.fire_click());
    assert!(ast_btn_mult.fire_click());

    // Step 3: Compute -> (190 * 3) - 10 = 560
    assert!(vm_btn_calc.fire_click());
    assert!(ast_btn_calc.fire_click());
    let f2_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let f2_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    let n2_vm = crate::stdlib::topia::object_to_node(&f2_vm);
    let n2_ast = crate::stdlib::topia::object_to_node(&f2_ast);
    assert_eq!(n2_vm.children()[0].as_text(), Some("Status: ACTIVE | Value: 560"));
    assert_eq!(n2_ast.children()[0].as_text(), Some("Status: ACTIVE | Value: 560"));

    // Step 4: Toggle -> PAUSED
    assert!(vm_btn_toggle.fire_click());
    assert!(ast_btn_toggle.fire_click());
    // Compute -> 560 + 1 = 561
    assert!(vm_btn_calc.fire_click());
    assert!(ast_btn_calc.fire_click());
    let f3_vm = crate::evaluator::apply_function(vm_vb, vec![]);
    let f3_ast = crate::evaluator::apply_function(ast_vb, vec![]);
    let n3_vm = crate::stdlib::topia::object_to_node(&f3_vm);
    let n3_ast = crate::stdlib::topia::object_to_node(&f3_ast);
    assert_eq!(n3_vm.children()[0].as_text(), Some("Status: PAUSED | Value: 561"));
    assert_eq!(n3_ast.children()[0].as_text(), Some("Status: PAUSED | Value: 561"));
    assert_eq!(n3_vm.children()[2].as_text(), Some("Logs: 5"));
    assert_eq!(n3_ast.children()[2].as_text(), Some("Logs: 5"));
}

#[test]
fn test_challenger_differential_short_circuit_logic_with_side_effects() {
    let script = r#"
        var side_effect_and = false
        var side_effect_or = false

        func trigger_and() {
            side_effect_and = true
            return true
        }

        func trigger_or() {
            side_effect_or = true
            return false
        }

        let and_res = false && trigger_and()
        let or_res = true || trigger_or()

        let out = [and_res, or_res, side_effect_and, side_effect_or]
        out
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on short circuit logic");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Boolean(false),
        Object::Boolean(true),
        Object::Boolean(false),
        Object::Boolean(false),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_type_annotations_runtime_enforcement() {
    let valid_script = r#"
        func add_typed(a: Int, b: Int) -> Int {
            return a + b
        }
        func greet_typed(name: String) -> String {
            return "Hi, " + name
        }
        let out = [add_typed(10, 20), greet_typed("Topia")]
        out
    "#;

    let expected_valid = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(30),
        Object::String("Hi, Topia".to_string()),
    ])));
    assert_eq!(run_ast(valid_script), expected_valid);
    assert_eq!(run_vm(valid_script).expect("VM failed on valid typed function"), expected_valid);

    let invalid_arg_script = r#"
        func add_typed(a: Int, b: Int) -> Int {
            return a + b
        }
        add_typed("not_int", 20)
    "#;
    assert!(matches!(run_ast(invalid_arg_script), Object::Error(_)));
    assert!(run_vm(invalid_arg_script).is_err() || matches!(run_vm(invalid_arg_script).unwrap(), Object::Error(_)));
}

#[test]
fn test_challenger_differential_stdlib_math_json_fs_time_os_in_vm() {
    let script = r#"
        let math = import("math")
        let json = import("json")
        let time = import("time")

        let sqrt_16 = math.sqrt(16)
        let pow_2_4 = math.pow(2, 4)
        let floor_3_7 = math.floor(3.7)

        let payload = {"title": "Topia", "version": 1, "features": ["ast", "vm"]}
        let serialized = json.stringify(payload)
        let deserialized = json.parse(serialized)

        let is_same_title = deserialized["title"] == "Topia"
        let is_same_ver = deserialized["version"] == 1

        let now_ts = time.now_ms()
        let valid_ts = now_ts > 0

        let out = [sqrt_16, pow_2_4, floor_3_7, is_same_title, is_same_ver, valid_ts]
        out
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on stdlib modules");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Float(4.0),
        Object::Integer(16),
        Object::Float(3.0),
        Object::Boolean(true),
        Object::Boolean(true),
        Object::Boolean(true),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_rapid_stress_1000_iterations() {
    let script = r#"
        let topia = import("topia")

        var total = 0
        let btn = topia.Button("Add", func() {
            total += 1
        })

        for i in 0..1000 {
            btn.on_click()
        }

        total
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on 1000 iterations stress");

    assert_eq!(ast_res, Object::Integer(1000));
    assert_eq!(vm_res, Object::Integer(1000));
}

#[test]
fn test_challenger_differential_matrix_transformation_pipeline() {
    let script = r#"
        var matrix = [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]

        func transform_matrix(mat, scale_fn) {
            for r in 0..3 {
                for c in 0..3 {
                    mat[r][c] = scale_fn(mat[r][c])
                }
            }
            return mat
        }

        let scaled = transform_matrix(matrix, func(val) { return val * 10 })
        let flattened = [
            scaled[0][0], scaled[0][1], scaled[0][2],
            scaled[1][0], scaled[1][1], scaled[1][2],
            scaled[2][0], scaled[2][1], scaled[2][2]
        ]
        flattened
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on matrix transformation");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(10), Object::Integer(20), Object::Integer(30),
        Object::Integer(40), Object::Integer(50), Object::Integer(60),
        Object::Integer(70), Object::Integer(80), Object::Integer(90),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_deep_lexical_scoping_and_shadowing() {
    let script = r#"
        var global_x = 100

        func level1(a) {
            var local_1 = a * 2
            return func(b) {
                var local_2 = local_1 + b
                return func(c) {
                    global_x += (local_2 + c)
                    return global_x
                }
            }
        }

        let l1 = level1(5)       // local_1 = 10
        let l2 = l1(20)          // local_2 = 30
        let r1 = l2(3)           // global_x = 100 + 33 = 133
        let r2 = l2(7)           // global_x = 133 + 37 = 170

        let out = [r1, r2, global_x]
        out
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on deep lexical scoping");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(133),
        Object::Integer(170),
        Object::Integer(170),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_dynamic_ui_variant_transitions() {
    let script = r#"
        let topia = import("topia")
        var mode = 0 // 0: Empty, 1: Text, 2: Button, 3: Full UI
        var score = 0

        let btn_cycle = topia.Button("Cycle", func() {
            mode = (mode + 1) % 4
        })

        let btn_score = topia.Button("Score +10", func() {
            score += 10
        })

        let view_builder = func() {
            if mode == 0 {
                return topia.Empty()
            } else {
                if mode == 1 {
                    return topia.Text("Mode 1: " + score)
                } else {
                    if mode == 2 {
                        return btn_score
                    } else {
                        return topia.VStack([
                            topia.Text("Score: " + score),
                            topia.HStack([btn_cycle, btn_score])
                        ])
                    }
                }
            }
        }

        let pack = [btn_cycle, btn_score, view_builder]
        pack
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on dynamic UI variant script");

    let vm_pack = match vm_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };
    let ast_pack = match ast_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };

    let mut vm_btn_cycle = crate::stdlib::topia::object_to_node(&vm_pack[0]);
    let mut vm_btn_score = crate::stdlib::topia::object_to_node(&vm_pack[1]);
    let vm_vb = vm_pack[2].clone();

    let mut ast_btn_cycle = crate::stdlib::topia::object_to_node(&ast_pack[0]);
    let mut ast_btn_score = crate::stdlib::topia::object_to_node(&ast_pack[1]);
    let ast_vb = ast_pack[2].clone();

    // Mode 0: Empty
    let v0_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let v0_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    assert!(matches!(crate::stdlib::topia::object_to_node(&v0_vm), topia::Node::Empty));
    assert!(matches!(crate::stdlib::topia::object_to_node(&v0_ast), topia::Node::Empty));

    // Cycle -> Mode 1: Text
    assert!(vm_btn_cycle.fire_click());
    assert!(ast_btn_cycle.fire_click());
    let v1_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let v1_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    let n1_vm = crate::stdlib::topia::object_to_node(&v1_vm);
    let n1_ast = crate::stdlib::topia::object_to_node(&v1_ast);
    assert_eq!(n1_vm.as_text(), Some("Mode 1: 0"));
    assert_eq!(n1_ast.as_text(), Some("Mode 1: 0"));

    // Cycle -> Mode 2: Button
    assert!(vm_btn_cycle.fire_click());
    assert!(ast_btn_cycle.fire_click());
    let v2_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let v2_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    let n2_vm = crate::stdlib::topia::object_to_node(&v2_vm);
    let n2_ast = crate::stdlib::topia::object_to_node(&v2_ast);
    assert_eq!(n2_vm.as_button_label(), Some("Score +10"));
    assert_eq!(n2_ast.as_button_label(), Some("Score +10"));

    // Click score twice
    assert!(vm_btn_score.fire_click());
    assert!(vm_btn_score.fire_click());
    assert!(ast_btn_score.fire_click());
    assert!(ast_btn_score.fire_click());

    // Cycle -> Mode 3: Full UI
    assert!(vm_btn_cycle.fire_click());
    assert!(ast_btn_cycle.fire_click());
    let v3_vm = crate::evaluator::apply_function(vm_vb.clone(), vec![]);
    let v3_ast = crate::evaluator::apply_function(ast_vb.clone(), vec![]);
    let n3_vm = crate::stdlib::topia::object_to_node(&v3_vm);
    let n3_ast = crate::stdlib::topia::object_to_node(&v3_ast);
    assert_eq!(n3_vm.children()[0].as_text(), Some("Score: 20"));
    assert_eq!(n3_ast.children()[0].as_text(), Some("Score: 20"));
    assert_eq!(n3_vm.children()[1].child_count(), 2);
    assert_eq!(n3_ast.children()[1].child_count(), 2);
}

#[test]
fn test_challenger_differential_hash_heterogeneous_keys_parity() {
    let script = r#"
        var dict = {
            "str_key": 100,
            123: "int_key",
            true: "bool_key"
        }

        dict["str_key"] += 50
        dict[123] = "updated_int"
        dict[false] = "new_bool"

        let out = [dict["str_key"], dict[123], dict[true], dict[false]]
        out
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on heterogeneous hash keys");

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(150),
        Object::String("updated_int".to_string()),
        Object::String("bool_key".to_string()),
        Object::String("new_bool".to_string()),
    ])));

    assert_eq!(ast_res, expected);
    assert_eq!(vm_res, expected);
}

#[test]
fn test_challenger_differential_error_semantics_parity() {
    // 1. Division by zero (Int)
    let div_zero_int = "10 / 0";
    assert_eq!(run_ast(div_zero_int), Object::Error("division by zero".to_string()));
    assert_eq!(run_vm(div_zero_int).unwrap_err(), "division by zero");

    // 2. Division by zero (Float)
    let div_zero_float = "10.0 / 0.0";
    assert_eq!(run_ast(div_zero_float), Object::Error("division by zero".to_string()));
    assert_eq!(run_vm(div_zero_float).unwrap_err(), "division by zero");

    // 3. Modulo by zero
    let mod_zero = "10 % 0";
    assert_eq!(run_ast(mod_zero), Object::Error("division by zero".to_string()));
    assert_eq!(run_vm(mod_zero).unwrap_err(), "division by zero");

    // 4. Calling non-callable
    let non_callable = "let x = 42\nx(10)";
    assert!(matches!(run_ast(non_callable), Object::Error(_)));
    assert!(run_vm(non_callable).is_err());
}

// =============================================================================
// M4 EMPIRICAL CHALLENGER 1: E2E DEMO SCRIPT & ADVERSARIAL VARIATIONS SUITE
// =============================================================================

#[test]
fn test_m4_empirical_demo_file_e2e_parity() {
    let script = r#"
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
            topia.VStack([
                topia.Text("Topia Counter Demo"),
                topia.Text("Count: " + count),
                topia.HStack([btn_dec, btn_inc, btn_reset])
            ])
        }
        let pack = [app, btn_dec, btn_inc, btn_reset, view]
        pack
    "#;

    let ast_pack_obj = run_ast(script);
    let vm_pack_obj = run_vm(script).expect("VM failed on topia_demo script");

    let ast_pack = match ast_pack_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };
    let vm_pack = match vm_pack_obj {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array"),
    };

    let mut ast_dec = crate::stdlib::topia::object_to_node(&ast_pack[1]);
    let mut ast_inc = crate::stdlib::topia::object_to_node(&ast_pack[2]);
    let mut ast_reset = crate::stdlib::topia::object_to_node(&ast_pack[3]);
    let ast_view_fn = ast_pack[4].clone();

    let mut vm_dec = crate::stdlib::topia::object_to_node(&vm_pack[1]);
    let mut vm_inc = crate::stdlib::topia::object_to_node(&vm_pack[2]);
    let mut vm_reset = crate::stdlib::topia::object_to_node(&vm_pack[3]);
    let vm_view_fn = vm_pack[4].clone();

    // 1. Initial State Check (count = 0)
    let ast_v0 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v0 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n0 = crate::stdlib::topia::object_to_node(&ast_v0);
    let vm_n0 = crate::stdlib::topia::object_to_node(&vm_v0);

    assert_eq!(ast_n0.children()[0].as_text(), Some("Topia Counter Demo"));
    assert_eq!(vm_n0.children()[0].as_text(), Some("Topia Counter Demo"));
    assert_eq!(ast_n0.children()[1].as_text(), Some("Count: 0"));
    assert_eq!(vm_n0.children()[1].as_text(), Some("Count: 0"));
    assert_eq!(ast_n0.children()[2].child_count(), 3);
    assert_eq!(vm_n0.children()[2].child_count(), 3);

    // 2. Increment twice (count -> 2)
    assert!(ast_inc.fire_click());
    assert!(ast_inc.fire_click());
    assert!(vm_inc.fire_click());
    assert!(vm_inc.fire_click());

    let ast_v1 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v1 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n1 = crate::stdlib::topia::object_to_node(&ast_v1);
    let vm_n1 = crate::stdlib::topia::object_to_node(&vm_v1);

    assert_eq!(ast_n1.children()[1].as_text(), Some("Count: 2"));
    assert_eq!(vm_n1.children()[1].as_text(), Some("Count: 2"));

    // 3. Decrement once (count -> 1)
    assert!(ast_dec.fire_click());
    assert!(vm_dec.fire_click());

    let ast_v2 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v2 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n2 = crate::stdlib::topia::object_to_node(&ast_v2);
    let vm_n2 = crate::stdlib::topia::object_to_node(&vm_v2);

    assert_eq!(ast_n2.children()[1].as_text(), Some("Count: 1"));
    assert_eq!(vm_n2.children()[1].as_text(), Some("Count: 1"));

    // 4. Reset (count -> 0)
    assert!(ast_reset.fire_click());
    assert!(vm_reset.fire_click());

    let ast_v3 = crate::evaluator::apply_function(ast_view_fn, vec![]);
    let vm_v3 = crate::evaluator::apply_function(vm_view_fn, vec![]);
    let ast_n3 = crate::stdlib::topia::object_to_node(&ast_v3);
    let vm_n3 = crate::stdlib::topia::object_to_node(&vm_v3);

    assert_eq!(ast_n3.children()[1].as_text(), Some("Count: 0"));
    assert_eq!(vm_n3.children()[1].as_text(), Some("Count: 0"));
}

#[test]
fn test_m4_empirical_adversarial_counter_advanced_parity() {
    let script = r#"
        let topia = import("topia")
        var count = 0
        var step = 1
        var status = "Normal"

        let btn_dec = topia.Button("- Step", func() {
            count = count - step
            status = "Decremented"
        })

        let btn_inc = topia.Button("+ Step", func() {
            count = count + step
            status = "Incremented"
        })

        let btn_step_up = topia.Button("Step +1", func() {
            step = step + 1
            status = "Step Increased"
        })

        let btn_step_down = topia.Button("Step -1", func() {
            if step > 1 {
                step = step - 1
                status = "Step Decreased"
            } else {
                status = "Step Minimum Reached"
            }
        })

        let btn_double = topia.Button("Double (*2)", func() {
            count = count * 2
            status = "Doubled"
        })

        let btn_reset = topia.Button("Reset All", func() {
            count = 0
            step = 1
            status = "Reset to Default"
        })

        let view = func() {
            topia.VStack([
                topia.Text("=== Advanced Reactive Counter ==="),
                topia.Text("Current Count: " + count),
                topia.Text("Step Size: " + step),
                topia.Text("Status: " + status),
                topia.HStack([btn_dec, btn_inc], 10.0),
                topia.HStack([btn_step_down, btn_step_up], 10.0),
                topia.HStack([btn_double, btn_reset], 10.0)
            ], 8.0)
        }

        let pack = [btn_dec, btn_inc, btn_step_up, btn_step_down, btn_double, btn_reset, view]
        pack
    "#;

    let ast_pack_obj = run_ast(script);
    let vm_pack_obj = run_vm(script).expect("VM failed on advanced counter script");

    let ast_pack = match ast_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };
    let vm_pack = match vm_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };

    let mut ast_btns: Vec<topia::Node> = ast_pack[0..6].iter().map(crate::stdlib::topia::object_to_node).collect();
    let mut vm_btns: Vec<topia::Node> = vm_pack[0..6].iter().map(crate::stdlib::topia::object_to_node).collect();
    let ast_view_fn = ast_pack[6].clone();
    let vm_view_fn = vm_pack[6].clone();

    // Initial check
    let ast_v0 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v0 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n0 = crate::stdlib::topia::object_to_node(&ast_v0);
    let vm_n0 = crate::stdlib::topia::object_to_node(&vm_v0);
    assert_eq!(ast_n0.children()[1].as_text(), Some("Current Count: 0"));
    assert_eq!(vm_n0.children()[1].as_text(), Some("Current Count: 0"));
    assert_eq!(ast_n0.children()[2].as_text(), Some("Step Size: 1"));
    assert_eq!(vm_n0.children()[2].as_text(), Some("Step Size: 1"));
    assert_eq!(ast_n0.children()[3].as_text(), Some("Status: Normal"));
    assert_eq!(vm_n0.children()[3].as_text(), Some("Status: Normal"));

    // 1. Click + Step (count = 1)
    ast_btns[1].fire_click();
    vm_btns[1].fire_click();

    // 2. Click Step +1 (step = 2)
    ast_btns[2].fire_click();
    vm_btns[2].fire_click();

    // 3. Click + Step (count = 3)
    ast_btns[1].fire_click();
    vm_btns[1].fire_click();

    // 4. Click Double (count = 6)
    ast_btns[4].fire_click();
    vm_btns[4].fire_click();

    let ast_v1 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v1 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n1 = crate::stdlib::topia::object_to_node(&ast_v1);
    let vm_n1 = crate::stdlib::topia::object_to_node(&vm_v1);
    assert_eq!(ast_n1.children()[1].as_text(), Some("Current Count: 6"));
    assert_eq!(vm_n1.children()[1].as_text(), Some("Current Count: 6"));
    assert_eq!(ast_n1.children()[2].as_text(), Some("Step Size: 2"));
    assert_eq!(vm_n1.children()[2].as_text(), Some("Step Size: 2"));
    assert_eq!(ast_n1.children()[3].as_text(), Some("Status: Doubled"));
    assert_eq!(vm_n1.children()[3].as_text(), Some("Status: Doubled"));

    // 5. Click Reset
    ast_btns[5].fire_click();
    vm_btns[5].fire_click();

    let ast_v2 = crate::evaluator::apply_function(ast_view_fn, vec![]);
    let vm_v2 = crate::evaluator::apply_function(vm_view_fn, vec![]);
    let ast_n2 = crate::stdlib::topia::object_to_node(&ast_v2);
    let vm_n2 = crate::stdlib::topia::object_to_node(&vm_v2);
    assert_eq!(ast_n2.children()[1].as_text(), Some("Current Count: 0"));
    assert_eq!(vm_n2.children()[1].as_text(), Some("Current Count: 0"));
    assert_eq!(ast_n2.children()[2].as_text(), Some("Step Size: 1"));
    assert_eq!(vm_n2.children()[2].as_text(), Some("Step Size: 1"));
    assert_eq!(ast_n2.children()[3].as_text(), Some("Status: Reset to Default"));
    assert_eq!(vm_n2.children()[3].as_text(), Some("Status: Reset to Default"));
}

#[test]
fn test_m4_empirical_adversarial_nested_dashboard_parity() {
    let script = r#"
        let topia = import("topia")
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
            let header = topia.HStack([
                topia.Text("System Dashboard"),
                topia.Text("Active: " + active_tab),
                topia.Text("Alerts: " + notifications)
            ], 15.0)

            let nav_bar = topia.HStack([
                tab_home,
                tab_analytics,
                tab_settings,
                btn_clear_notif
            ], 8.0)

            var body_content = topia.Empty()
            if active_tab == "Home" {
                body_content = topia.VStack([
                    topia.Text("Welcome to Home View"),
                    topia.Text("Total tab navigation switches: " + tab_visits)
                ], 5.0)
            } else {
                if active_tab == "Analytics" {
                    body_content = topia.VStack([
                        topia.Text("Analytics & Metrics"),
                        topia.Text("System load: Optimal"),
                        topia.Text("Visits: " + tab_visits)
                    ], 5.0)
                } else {
                    body_content = topia.VStack([
                        topia.Text("User Settings Panel"),
                        topia.Text("Notification Count: " + notifications)
                    ], 5.0)
                }
            }

            return topia.VStack([
                header,
                topia.Text("----------------------------------------"),
                nav_bar,
                topia.Text("----------------------------------------"),
                body_content
            ], 10.0)
        }

        let pack = [tab_home, tab_analytics, tab_settings, btn_clear_notif, view]
        pack
    "#;

    let ast_pack_obj = run_ast(script);
    let vm_pack_obj = run_vm(script).expect("VM failed on nested dashboard");

    let ast_pack = match ast_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };
    let vm_pack = match vm_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };

    let mut ast_btns: Vec<topia::Node> = ast_pack[0..4].iter().map(crate::stdlib::topia::object_to_node).collect();
    let mut vm_btns: Vec<topia::Node> = vm_pack[0..4].iter().map(crate::stdlib::topia::object_to_node).collect();
    let ast_view_fn = ast_pack[4].clone();
    let vm_view_fn = vm_pack[4].clone();

    // 1. Initial Home Tab
    let ast_v0 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v0 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n0 = crate::stdlib::topia::object_to_node(&ast_v0);
    let vm_n0 = crate::stdlib::topia::object_to_node(&vm_v0);
    assert_eq!(ast_n0.children()[4].children()[0].as_text(), Some("Welcome to Home View"));
    assert_eq!(vm_n0.children()[4].children()[0].as_text(), Some("Welcome to Home View"));

    // 2. Switch to Analytics Tab
    ast_btns[1].fire_click();
    vm_btns[1].fire_click();

    let ast_v1 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v1 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n1 = crate::stdlib::topia::object_to_node(&ast_v1);
    let vm_n1 = crate::stdlib::topia::object_to_node(&vm_v1);
    assert_eq!(ast_n1.children()[4].children()[0].as_text(), Some("Analytics & Metrics"));
    assert_eq!(vm_n1.children()[4].children()[0].as_text(), Some("Analytics & Metrics"));
    assert_eq!(ast_n1.children()[4].children()[2].as_text(), Some("Visits: 1"));
    assert_eq!(vm_n1.children()[4].children()[2].as_text(), Some("Visits: 1"));

    // 3. Switch to Settings Tab and Clear Alerts
    ast_btns[2].fire_click();
    vm_btns[2].fire_click();
    ast_btns[3].fire_click();
    vm_btns[3].fire_click();

    let ast_v2 = crate::evaluator::apply_function(ast_view_fn, vec![]);
    let vm_v2 = crate::evaluator::apply_function(vm_view_fn, vec![]);
    let ast_n2 = crate::stdlib::topia::object_to_node(&ast_v2);
    let vm_n2 = crate::stdlib::topia::object_to_node(&vm_v2);
    assert_eq!(ast_n2.children()[4].children()[0].as_text(), Some("User Settings Panel"));
    assert_eq!(vm_n2.children()[4].children()[0].as_text(), Some("User Settings Panel"));
    assert_eq!(ast_n2.children()[4].children()[1].as_text(), Some("Notification Count: 0"));
    assert_eq!(vm_n2.children()[4].children()[1].as_text(), Some("Notification Count: 0"));
    assert_eq!(ast_n2.children()[0].children()[2].as_text(), Some("Alerts: 0"));
    assert_eq!(vm_n2.children()[0].children()[2].as_text(), Some("Alerts: 0"));
}

#[test]
fn test_m4_empirical_adversarial_todo_manager_parity() {
    let script = r#"
        let topia = import("topia")
        var todos = ["Learn Topia", "Build f(x) UI"]
        var last_action = "Initial state"

        let btn_add = topia.Button("Add Item", func() {
            push(todos, "Task " + (len(todos) + 1))
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

            let items_stack = topia.VStack(item_nodes, 4.0)
            let action_bar = topia.HStack([btn_add, btn_pop, btn_clear], 8.0)

            return topia.VStack([
                topia.Text("=== Dynamic Todo List ==="),
                topia.Text("Total Tasks: " + len(todos)),
                topia.Text("Status: " + last_action),
                items_stack,
                action_bar
            ], 6.0)
        }

        let pack = [btn_add, btn_pop, btn_clear, view]
        pack
    "#;

    let ast_pack_obj = run_ast(script);
    let vm_pack_obj = run_vm(script).expect("VM failed on todo manager");

    let ast_pack = match ast_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };
    let vm_pack = match vm_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };

    let mut ast_btns: Vec<topia::Node> = ast_pack[0..3].iter().map(crate::stdlib::topia::object_to_node).collect();
    let mut vm_btns: Vec<topia::Node> = vm_pack[0..3].iter().map(crate::stdlib::topia::object_to_node).collect();
    let ast_view_fn = ast_pack[3].clone();
    let vm_view_fn = vm_pack[3].clone();

    // 1. Initial 2 items
    let ast_v0 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v0 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n0 = crate::stdlib::topia::object_to_node(&ast_v0);
    let vm_n0 = crate::stdlib::topia::object_to_node(&vm_v0);
    assert_eq!(ast_n0.children()[3].child_count(), 2);
    assert_eq!(vm_n0.children()[3].child_count(), 2);

    // 2. Add two items
    ast_btns[0].fire_click();
    vm_btns[0].fire_click();
    ast_btns[0].fire_click();
    vm_btns[0].fire_click();

    let ast_v1 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v1 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n1 = crate::stdlib::topia::object_to_node(&ast_v1);
    let vm_n1 = crate::stdlib::topia::object_to_node(&vm_v1);
    assert_eq!(ast_n1.children()[1].as_text(), Some("Total Tasks: 4"));
    assert_eq!(vm_n1.children()[1].as_text(), Some("Total Tasks: 4"));
    assert_eq!(ast_n1.children()[3].child_count(), 4);
    assert_eq!(vm_n1.children()[3].child_count(), 4);

    // 3. Remove one item
    ast_btns[1].fire_click();
    vm_btns[1].fire_click();

    let ast_v2 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v2 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n2 = crate::stdlib::topia::object_to_node(&ast_v2);
    let vm_n2 = crate::stdlib::topia::object_to_node(&vm_v2);
    assert_eq!(ast_n2.children()[1].as_text(), Some("Total Tasks: 3"));
    assert_eq!(vm_n2.children()[1].as_text(), Some("Total Tasks: 3"));
    assert_eq!(ast_n2.children()[3].child_count(), 3);
    assert_eq!(vm_n2.children()[3].child_count(), 3);

    // 4. Clear all
    ast_btns[2].fire_click();
    vm_btns[2].fire_click();

    let ast_v3 = crate::evaluator::apply_function(ast_view_fn, vec![]);
    let vm_v3 = crate::evaluator::apply_function(vm_view_fn, vec![]);
    let ast_n3 = crate::stdlib::topia::object_to_node(&ast_v3);
    let vm_n3 = crate::stdlib::topia::object_to_node(&vm_v3);
    assert_eq!(ast_n3.children()[1].as_text(), Some("Total Tasks: 0"));
    assert_eq!(vm_n3.children()[1].as_text(), Some("Total Tasks: 0"));
    assert_eq!(ast_n3.children()[3].children()[0].as_text(), Some("(No tasks available)"));
    assert_eq!(vm_n3.children()[3].children()[0].as_text(), Some("(No tasks available)"));
}

#[test]
fn test_m4_empirical_adversarial_struct_state_ui_parity() {
    let script = r#"
        let topia = import("topia")

        struct UserProfile {
            name: String,
            level: Int,
            points: Int,
            is_vip: Bool
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
            topia.VStack([
                topia.Text("Player: " + user.name),
                topia.Text("Level: " + user.level),
                topia.Text("Points: " + user.points),
                topia.Text("VIP Status: " + user.is_vip),
                topia.HStack([btn_level_up, btn_spend, btn_reset_user], 8.0)
            ], 10.0)
        }

        let pack = [btn_level_up, btn_spend, btn_reset_user, view]
        pack
    "#;

    let ast_pack_obj = run_ast(script);
    let vm_pack_obj = run_vm(script).expect("VM failed on struct state UI");

    let ast_pack = match ast_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };
    let vm_pack = match vm_pack_obj { Object::Array(rc) => rc.borrow().clone(), _ => panic!() };

    let mut ast_btns: Vec<topia::Node> = ast_pack[0..3].iter().map(crate::stdlib::topia::object_to_node).collect();
    let mut vm_btns: Vec<topia::Node> = vm_pack[0..3].iter().map(crate::stdlib::topia::object_to_node).collect();
    let ast_view_fn = ast_pack[3].clone();
    let vm_view_fn = vm_pack[3].clone();

    // 1. Initial State
    let ast_v0 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v0 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n0 = crate::stdlib::topia::object_to_node(&ast_v0);
    let vm_n0 = crate::stdlib::topia::object_to_node(&vm_v0);
    assert_eq!(ast_n0.children()[1].as_text(), Some("Level: 1"));
    assert_eq!(vm_n0.children()[1].as_text(), Some("Level: 1"));
    assert_eq!(ast_n0.children()[2].as_text(), Some("Points: 100"));
    assert_eq!(vm_n0.children()[2].as_text(), Some("Points: 100"));
    assert_eq!(ast_n0.children()[3].as_text(), Some("VIP Status: false"));
    assert_eq!(vm_n0.children()[3].as_text(), Some("VIP Status: false"));

    // 2. Level up 4 times to reach VIP (level 5, points 300)
    for _ in 0..4 {
        ast_btns[0].fire_click();
        vm_btns[0].fire_click();
    }

    let ast_v1 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v1 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n1 = crate::stdlib::topia::object_to_node(&ast_v1);
    let vm_n1 = crate::stdlib::topia::object_to_node(&vm_v1);
    assert_eq!(ast_n1.children()[1].as_text(), Some("Level: 5"));
    assert_eq!(vm_n1.children()[1].as_text(), Some("Level: 5"));
    assert_eq!(ast_n1.children()[2].as_text(), Some("Points: 300"));
    assert_eq!(vm_n1.children()[2].as_text(), Some("Points: 300"));
    assert_eq!(ast_n1.children()[3].as_text(), Some("VIP Status: true"));
    assert_eq!(vm_n1.children()[3].as_text(), Some("VIP Status: true"));

    // 3. Spend points twice (300 -> 240)
    ast_btns[1].fire_click();
    vm_btns[1].fire_click();
    ast_btns[1].fire_click();
    vm_btns[1].fire_click();

    let ast_v2 = crate::evaluator::apply_function(ast_view_fn.clone(), vec![]);
    let vm_v2 = crate::evaluator::apply_function(vm_view_fn.clone(), vec![]);
    let ast_n2 = crate::stdlib::topia::object_to_node(&ast_v2);
    let vm_n2 = crate::stdlib::topia::object_to_node(&vm_v2);
    assert_eq!(ast_n2.children()[2].as_text(), Some("Points: 240"));
    assert_eq!(vm_n2.children()[2].as_text(), Some("Points: 240"));

    // 4. Reset
    ast_btns[2].fire_click();
    vm_btns[2].fire_click();

    let ast_v3 = crate::evaluator::apply_function(ast_view_fn, vec![]);
    let vm_v3 = crate::evaluator::apply_function(vm_view_fn, vec![]);
    let ast_n3 = crate::stdlib::topia::object_to_node(&ast_v3);
    let vm_n3 = crate::stdlib::topia::object_to_node(&vm_v3);
    assert_eq!(ast_n3.children()[1].as_text(), Some("Level: 1"));
    assert_eq!(vm_n3.children()[1].as_text(), Some("Level: 1"));
    assert_eq!(ast_n3.children()[2].as_text(), Some("Points: 100"));
    assert_eq!(vm_n3.children()[2].as_text(), Some("Points: 100"));
    assert_eq!(ast_n3.children()[3].as_text(), Some("VIP Status: false"));
    assert_eq!(vm_n3.children()[3].as_text(), Some("VIP Status: false"));
}

#[test]
fn test_m4_empirical_rapid_callback_stress_500_iterations() {
    let script = r#"
        let topia = import("topia")
        var sum = 0
        let btn = topia.Button("+1", func() {
            sum += 1
        })
        for i in 0..500 {
            btn.on_click()
        }
        sum
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on 500 callback stress");

    assert_eq!(ast_res, Object::Integer(500));
    assert_eq!(vm_res, Object::Integer(500));
}

#[test]
fn test_m4_empirical_deeply_nested_layout_and_headless_render() {
    let script = r#"
        let topia = import("topia")
        var root = topia.Text("Leaf")
        for i in 0..20 {
            if i % 2 == 0 {
                root = topia.VStack([root], 2.0)
            } else {
                root = topia.HStack([root], 2.0)
            }
        }
        root
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on deep nesting");

    let ast_node = crate::stdlib::topia::object_to_node(&ast_res);
    let vm_node = crate::stdlib::topia::object_to_node(&vm_res);

    // Verify tree depth
    fn compute_depth(node: &topia::Node) -> usize {
        match node {
            topia::Node::VStack { children, .. } | topia::Node::HStack { children, .. } => {
                1 + children.iter().map(compute_depth).max().unwrap_or(0)
            }
            _ => 1,
        }
    }

    assert_eq!(compute_depth(&ast_node), 21);
    assert_eq!(compute_depth(&vm_node), 21);
}

// =============================================================================
// M4 EMPIRICAL CHALLENGER 2: TIER 5 ADVERSARIAL COVERAGE HARDENING SUITE
// =============================================================================

#[test]
fn test_tier5_topia_callback_error_resilience_and_host_safety() {
    let script = r#"
        let topia = import("topia")
        var log = "start"

        // 1. Button callback with intentional runtime error
        let err_btn = topia.Button("Crash", func() {
            let arr = [1, 2]
            arr[100] = 999 // array index out of bounds error
        })

        // 2. Safe button callback
        let safe_btn = topia.Button("Safe", func() {
            log = "safe_executed"
        })

        let pack = [err_btn, safe_btn]
        pack
    "#;

    let eval_res = run_ast(script);
    let pack = match eval_res {
        Object::Array(rc) => rc.borrow().clone(),
        _ => panic!("Expected array pack"),
    };

    let mut err_node = crate::stdlib::topia::object_to_node(&pack[0]);
    let mut safe_node = crate::stdlib::topia::object_to_node(&pack[1]);

    // Firing error button must not panic or abort host execution
    assert!(err_node.fire_click());

    // Safe button must execute normally
    assert!(safe_node.fire_click());
}

#[test]
fn test_tier5_topia_multi_threaded_dual_engine_execution() {
    use std::thread;

    let mut handles = Vec::new();

    // Spawn 8 concurrent threads executing AST and VM scripts simultaneously
    for thread_id in 0..8 {
        let handle = thread::spawn(move || {
            let script = format!(r#"
                let topia = import("topia")
                var count = {}
                let btn = topia.Button("Add", func() {{
                    count += 10
                }})
                btn.on_click()
                btn.on_click()
                let view = func() {{
                    return topia.VStack([topia.Text("Thread: " + count)])
                }}
                let out = [count, view]
                out
            "#, thread_id * 100);

            // Run in AST mode
            let ast_res = run_ast(&script);
            let ast_arr = match ast_res {
                Object::Array(rc) => rc.borrow().clone(),
                _ => panic!("Expected array from thread {}", thread_id),
            };
            assert_eq!(ast_arr[0], Object::Integer((thread_id * 100) + 20));

            // Run in VM mode
            let vm_res = run_vm(&script).expect("VM failed in multithreaded test");
            let vm_arr = match vm_res {
                Object::Array(rc) => rc.borrow().clone(),
                _ => panic!("Expected array from thread {}", thread_id),
            };
            assert_eq!(vm_arr[0], Object::Integer((thread_id * 100) + 20));
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Multithreaded Topia test panicked");
    }
}

#[test]
fn test_tier5_topia_dynamic_closure_rebinding_and_state_swap() {
    let script = r#"
        let topia = import("topia")

        var dynamic_state = 100

        let btn_to_arr = topia.Button("To Array", func() {
            dynamic_state = [1, 2, 3]
        })

        let btn_to_hash = topia.Button("To Hash", func() {
            dynamic_state = {"status": "ok", "val": 42}
        })

        let btn_to_string = topia.Button("To String", func() {
            dynamic_state = "finished"
        })

        btn_to_arr.on_click()
        let res1 = len(dynamic_state)

        btn_to_hash.on_click()
        let res2 = dynamic_state.val

        btn_to_string.on_click()
        let res3 = dynamic_state

        let out = [res1, res2, res3]
        out
    "#;

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(3),
        Object::Integer(42),
        Object::String("finished".to_string()),
    ])));

    assert_eq!(run_ast(script), expected);
    assert_eq!(run_vm(script).expect("VM failed on dynamic state swap"), expected);
}

#[test]
fn test_tier5_topia_recursive_and_curried_view_builders() {
    let script = r#"
        let topia = import("topia")

        func make_builder(prefix) {
            return func(count) {
                return topia.VStack([
                    topia.Text(prefix + ": " + count),
                    topia.Button("Click", func() {})
                ])
            }
        }

        let builder_a = make_builder("Section A")
        let builder_b = make_builder("Section B")

        let node_a_obj = builder_a(10)
        let node_b_obj = builder_b(20)

        let root = topia.HStack([node_a_obj, node_b_obj])
        root
    "#;

    let ast_res = run_ast(script);
    let vm_res = run_vm(script).expect("VM failed on curried view builder");

    let ast_node = crate::stdlib::topia::object_to_node(&ast_res);
    let vm_node = crate::stdlib::topia::object_to_node(&vm_res);

    assert_eq!(ast_node.child_count(), 2);
    assert_eq!(vm_node.child_count(), 2);
    assert_eq!(vm_node.children()[0].child_count(), 2);
    assert_eq!(vm_node.children()[0].children()[0].as_text(), Some("Section A: 10"));
    assert_eq!(vm_node.children()[1].children()[0].as_text(), Some("Section B: 20"));
}

#[test]
fn test_tier5_topia_rapid_reentrancy_stress_2000_cycles() {
    let script = r#"
        let topia = import("topia")

        var a = 0
        var b = 0
        var c = 0

        let btn_a = topia.Button("A", func() { a += 1 })
        let btn_b = topia.Button("B", func() { b += 2 })
        let btn_c = topia.Button("C", func() { c += 3 })

        for i in 0..2000 {
            btn_a.on_click()
            btn_b.on_click()
            btn_c.on_click()
        }

        let out = [a, b, c]
        out
    "#;

    let expected = Object::Array(Rc::new(RefCell::new(vec![
        Object::Integer(2000),
        Object::Integer(4000),
        Object::Integer(6000),
    ])));

    assert_eq!(run_ast(script), expected);
    assert_eq!(run_vm(script).expect("VM failed on 2000 cycles stress"), expected);
}






