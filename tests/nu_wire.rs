use r3viewer_wire::{Frame, Origin, VERSION, Value, write_frame, write_magic};
use std::{
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
fn input(origin: Origin, values: Vec<Frame>, end: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_magic(&mut bytes).unwrap();
    write_frame(
        &mut bytes,
        &Frame::Start {
            version: VERSION,
            cwd: std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            origin,
        },
    )
    .unwrap();
    for value in values {
        write_frame(&mut bytes, &value).unwrap();
    }
    if end {
        write_frame(&mut bytes, &Frame::End).unwrap();
    }
    bytes
}
fn run(args: &[&str], bytes: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_r3viewer"))
        .args(["--nu-wire", "--inline"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn typed_strings_are_literal_and_control_safe() {
    let result = run(
        &[],
        &input(
            Origin::SingleValue,
            vec![Frame::Value(Value::String("notes.md\r\n\x1b[31m".into()))],
            true,
        ),
    );
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "notes.md\r\n␛[31m"
    );
}
#[test]
fn nothing_empty_list_and_empty_text_stay_distinct() {
    let values = [
        (Value::Nothing, "nothing"),
        (Value::List(vec![]), "list([])"),
        (Value::String(String::new()), ""),
    ];
    let outputs = values
        .into_iter()
        .map(|(value, _)| {
            run(
                &[],
                &input(Origin::SingleValue, vec![Frame::Value(value)], true),
            )
            .stdout
        })
        .collect::<Vec<_>>();
    assert_ne!(outputs[0], outputs[1]);
    assert_ne!(outputs[0], outputs[2]);
    assert_ne!(outputs[1], outputs[2]);
    assert!(outputs[2].is_empty());
}
#[test]
fn no_origin_and_incomplete_wire_fail_without_content() {
    for (bytes, code) in [
        (input(Origin::Absent, vec![], true), 2),
        (
            input(
                Origin::SingleValue,
                vec![Frame::Value(Value::Int(42))],
                false,
            ),
            1,
        ),
    ] {
        let result = run(&[], &bytes);
        assert_eq!(result.status.code(), Some(code));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}
#[test]
fn argument_conflict_rejects_start_without_waiting_for_end() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_r3viewer"))
        .args(["--nu-wire", "--inline", "--value", "argument"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut held = child.stdin.take().unwrap();
    held.write_all(&input(Origin::SingleValue, vec![], false))
        .unwrap();
    held.flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(2));
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("argument conflict waited for End/EOF");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(held);
    let output = child.wait_with_output().unwrap();
    assert!(output.stdout.is_empty());
}
#[test]
fn path_requires_a_single_string_and_literal_argument_accepts_absence() {
    let wrong = run(
        &["--path"],
        &input(
            Origin::SingleValue,
            vec![Frame::Value(Value::List(vec![Value::String(
                "notes.md".into(),
            )]))],
            true,
        ),
    );
    assert_eq!(wrong.status.code(), Some(2));
    assert!(wrong.stdout.is_empty());
    let literal = run(
        &["--value", "notes.md"],
        &input(Origin::Absent, vec![], true),
    );
    assert!(literal.status.success());
    assert_eq!(literal.stdout, b"notes.md");
}
