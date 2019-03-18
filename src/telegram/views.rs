use crate::models::{*, AttendanceStatus::*};
use crate::util::collections::*;

lazy_static! {
    pub static ref AVAILABLE_COMMANDS: String = render_available_commands();
}

fn render_available_commands() -> String {
    let cmds = vec![
        "start_roll_call",
        "end_roll_call",
        "set_title",
        "shh",
        "louder",
        "in",
        "out",
        "maybe",
        "set_in_for",
        "set_out_for",
        "set_maybe_for",
        "whos_in",
        "available_commands"
    ];

    let list = cmds
        .iter()
        .map(|cmd| format!(" 🍺 /{}", cmd))
        .collect::<Vec<_>>()
        .join("\n");

    format!("Available commands:\n{}", list)
}

pub fn render_announcement(username: &str, status: AttendanceStatus) -> String {
    match status {
        In => format!("{} is in!", username),
        Out => format!("{} is out!", username),
        Maybe => format!("{} might come!", username),
    }
}

pub fn render_responses(call_with_responses: &CallWithResponses) -> String {
    let (call, responses) = call_with_responses;

    if call.quiet {
        render_responses_short(responses)
    } else {
        render_responses_full(responses)
    }
}

pub fn render_responses_short(responses: &[RollCallResponse]) -> String {
    let responses_by_status = group_by(responses, |response| response.status);
    let count_by_status = map_values(responses_by_status, |responses| responses.len());

    use AttendanceStatus::*;
    let in_count = count_by_status.get(&In).unwrap_or(&0_usize);
    let out_count = count_by_status.get(&Out).unwrap_or(&0_usize);
    let maybe_count = count_by_status.get(&Maybe).unwrap_or(&0_usize);

    format!("Total: {} in, {} out, {} might come.", in_count, out_count, maybe_count)
}

pub fn render_responses_full(responses: &[RollCallResponse]) -> String {
    fn get_response_line(response: &RollCallResponse) -> String {
        let user_name = response.user_name.as_ref().map_or("", |s| s.as_str());
        match response.reason.as_ref() {
            Some(reason) if !reason.is_empty() => format!(" - {} ({})", user_name, reason),
            _ => format!(" - {}", user_name),
        }
    }

    fn get_status_line(status: AttendanceStatus, count: usize) -> String {
        match status {
            AttendanceStatus::In => format!("In ({})", count),
            AttendanceStatus::Out => format!("Out ({})", count),
            AttendanceStatus::Maybe => format!("Maybe ({})", count),
        }
    }

    let responses_by_status = group_by(responses, |response| response.status);
    let responses_by_status = map_values(responses_by_status, |mut value| {
        value.sort_by_key(|response| response.updated_at);
        value
    });

    use AttendanceStatus::*;
    let result: Vec<String> = vec![In, Out, Maybe]
        .into_iter()
        .filter(|status| responses_by_status.contains_key(&status))
        .map(|status| {
            let responses = &responses_by_status[&status];
            let count = responses.len();
            let response_lines = responses
                .iter()
                .map(|&res| get_response_line(res))
                .collect::<Vec<_>>()
                .join("\n");
            let status_line = get_status_line(status, count);
            format!("{}\n{}", status_line, response_lines)
        })
        .collect();

    if result.is_empty() {
        return "No responses yet. 😢".to_owned();
    }

    result.join("\n\n")
}

#[cfg(test)]
mod tests {
    use crate::util::testutil::factories::*;

    use super::*;

    #[test]
    fn test_cached_available_commands_contains_all_commands() {
        let actual = &AVAILABLE_COMMANDS;
        let cmds = vec![
            "/start_roll_call",
            "/end_roll_call",
            "/set_title",
            "/shh",
            "/louder",
            "/in",
            "/out",
            "/maybe",
            "/set_in_for",
            "/set_out_for",
            "/set_maybe_for",
            "/whos_in",
            "/available_commands"
        ];

        for cmd in cmds.iter() {
            assert!(actual.contains(cmd));
        }
    }

    #[test]
    fn test_render_available_commands_shows_some_beers() {
        let actual = &AVAILABLE_COMMANDS;
        assert!(actual.contains('🍺'));
    }

    #[test]
    fn test_render_announcement() {
        assert!(render_announcement("Henry", In).contains("Henry is in!"));
        assert!(render_announcement("David", Out).contains("David is out!"));
        assert!(render_announcement("Daniel", Maybe).contains("Daniel might come!"));
    }

    #[test]
    fn test_render_responses_short() {
        let expected = "Total: 2 in, 1 out, 1 might come.";
        let actual = render_responses_short(&create_responses());

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_render_responses_full() {
        let actual = render_responses_full(&create_responses());
        let without_space = actual.replace(char::is_whitespace, "");

        let expected = "In(2)-David(willcome)-Henry(alsowillcome)Out(1)-Daniel(won'tcome)Maybe(1)-Albert(mightcome)";
        assert_eq!(expected, without_space);
    }

    #[test]
    fn test_render_responses_for_quiet_call() {
        let call_with_response = (create_quiet_call(), create_responses());
        let actual = render_responses(&call_with_response);
        let expected = "Total: 2 in, 1 out, 1 might come.";

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_render_responses_for_non_quiet_call() {
        let call_with_response = (create_call(), create_responses());
        let actual = render_responses(&call_with_response);
        let without_space = actual.replace(char::is_whitespace, "");

        let expected = "In(2)-David(willcome)-Henry(alsowillcome)Out(1)-Daniel(won'tcome)Maybe(1)-Albert(mightcome)";
        assert_eq!(expected, without_space);
    }
}
