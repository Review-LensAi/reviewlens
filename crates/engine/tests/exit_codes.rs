use engine::error::EngineError;

fn map_error_to_exit_code(err: anyhow::Error) -> i32 {
    if let Some(engine_error) = err.downcast_ref::<EngineError>() {
        match engine_error {
            EngineError::Config(_) => 2,
            _ => 3,
        }
    } else {
        3
    }
}

#[test]
fn config_error_returns_exit_code_two() {
    let err: anyhow::Error = EngineError::Config("bad config".into()).into();
    assert_eq!(map_error_to_exit_code(err), 2);
}

#[test]
fn runtime_error_returns_exit_code_three() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io failure");
    let err: anyhow::Error = EngineError::Io(io_err).into();
    assert_eq!(map_error_to_exit_code(err), 3);
}
