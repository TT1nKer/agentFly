pub fn handle_echo(input: &str) -> String {
    format!("echo.pong: {}", input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_returns_input() {
        let result = handle_echo("hello world");
        assert_eq!(result, "echo.pong: hello world");
    }

    #[test]
    fn test_echo_empty() {
        let result = handle_echo("");
        assert_eq!(result, "echo.pong: ");
    }
}
