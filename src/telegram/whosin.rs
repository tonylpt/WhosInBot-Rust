use failure::Error;

use crate::db::Repository;
use crate::models::Attendance;
use crate::telegram::base_bot::ChatCommand;

use super::base_bot;
use super::commands::{
    Command::{self, *},
    CommandParseError::{self, *},
};
use super::views::*;

pub struct WhosInBot<'a> {
    token: &'a str,
    repository: Box<dyn Repository>,
}

impl<'a> WhosInBot<'a> {
    pub fn new(token: &str, repository: Box<dyn Repository>) -> WhosInBot {
        WhosInBot { token, repository }
    }

    pub fn run(&self) -> base_bot::BotResult {
        base_bot::run(self.token, |cmd| self.handle(cmd))
    }

    fn handle(&self, chat_command: ChatCommand) -> Result<Option<String>, Error> {
        match Command::from_chat(chat_command) {
            Ok(command) => self.handle_command(command).map(Some),
            Err(parse_error) => self.handle_parse_error(parse_error).map(Some),
        }
    }

    fn handle_command(&self, command: Command) -> Result<String, Error> {
        let response = match command {
            StartRollCall { chat_id, ref title } => {
                info!("Starting roll call with title '{}'", title);
                self.repository.create_call(chat_id, title)?;
                "Roll call started.".into()
            }

            EndRollCall { chat_id } => {
                info!("Ending roll call");
                match self.repository.end_call(chat_id)? {
                    None => "No roll call in progress.".into(),
                    Some(_) => "Roll call ended.".into(),
                }
            }

            UpdateTitle { chat_id, ref title } => {
                info!("Updating roll call title to '{}'", title);
                match self.repository.update_title(chat_id, title)? {
                    None => "No roll call in progress.".into(),
                    Some(_) => "Roll call title set.".into(),
                }
            }

            UpdateQuiet { chat_id, quiet } => {
                info!("Updating roll call quiet to '{}'", quiet);
                match (self.repository.update_quiet(chat_id, quiet)?, quiet) {
                    (None, _) => "No roll call in progress.".into(),
                    (Some(_), true) => "Ok fine, I'll be quiet. 🤐".into(),
                    (Some(ref call_with_responses), false) => {
                        format!("Sure. 😃\n\n{}", render_responses(call_with_responses))
                    }
                }
            }

            UpdateAttendanceSelf {
                chat_id,
                user_id,
                username,
                status,
                reason,
            } => {
                info!("Setting own attendance for {} to '{}'", username, status);
                let attendance = Attendance::new(status, reason);
                match self
                    .repository
                    .set_response(chat_id, user_id, &username, &attendance)?
                {
                    None => "No roll call in progress.".into(),
                    Some(ref call_with_responses) => {
                        let announcement = render_announcement(&username, status);
                        let responses = render_responses(call_with_responses);
                        format!("{}\n\n{}", announcement, responses)
                    }
                }
            }

            UpdateAttendanceFor {
                chat_id,
                username,
                status,
                reason,
            } => {
                info!("Setting attendance for {} to '{}'", username, status);
                let attendance = Attendance::new(status, reason);
                match self
                    .repository
                    .set_response_for(chat_id, &username, &attendance)?
                {
                    None => "No roll call in progress.".into(),
                    Some(ref call_with_responses) => {
                        let announcement = render_announcement(&username, status);
                        let responses = render_responses(call_with_responses);
                        format!("{}\n\n{}", announcement, responses)
                    }
                }
            }

            GetAllAttendances { chat_id } => {
                match self.repository.get_call_with_responses(chat_id)? {
                    None => "No roll call in progress.".into(),
                    Some((ref call, ref responses)) => {
                        format!("{}\n\n{}", call.title, render_responses_full(responses))
                    }
                }
            }

            ListAvailableCommands => AVAILABLE_COMMANDS.clone(),
        };

        Ok(response)
    }

    fn handle_parse_error(&self, parse_error: CommandParseError) -> Result<String, Error> {
        let response = match parse_error {
            MissingTitle => "Please provide a title.",
            MissingUsername => "Please provide the person's name.",
            InvalidCommand(_command) => "I don't understand that.",
        };

        Ok(response.into())
    }
}

#[cfg(test)]
mod tests {
    use mockers::{matchers::ANY, Scenario};

    use crate::models::*;
    use crate::util::testutil::factories::*;

    use super::*;

    #[test]
    fn handle_start_roll_call() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        scenario.expect(
            repo.create_call_call(2, arg!("call title"))
                .and_return(Ok(call)),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/start_roll_call".to_string(),
            command_params: "call title".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(Some("Roll call started.".to_string()), result.unwrap());
    }

    #[test]
    fn handle_end_roll_call() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = RollCall {
            status: CallStatus::Closed,
            ..create_call()
        };

        scenario.expect(repo.end_call_call(2).and_return(Ok(Some(call))));

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/end_roll_call".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(Some("Roll call ended.".to_string()), result.unwrap());
    }

    #[test]
    fn handle_end_roll_call_no_in_progress() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        scenario.expect(repo.end_call_call(2).and_return(Ok(None)));

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/end_roll_call".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("No roll call in progress.".to_string()),
            result.unwrap()
        );
    }

    #[test]
    fn handle_update_title() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = RollCall {
            title: "new title".to_string(),
            ..create_call()
        };

        scenario.expect(
            repo.update_title_call(2, arg!("new title"))
                .and_return(Ok(Some(call))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_title".to_string(),
            command_params: "new title".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(Some("Roll call title set.".to_string()), result.unwrap());
    }

    #[test]
    fn handle_update_title_empty_title() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_title".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(Some("Please provide a title.".to_string()), result.unwrap());
    }

    #[test]
    fn handle_update_title_no_in_progress() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        scenario.expect(
            repo.update_title_call(2, arg!("new title"))
                .and_return(Ok(None)),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_title".to_string(),
            command_params: "new title".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("No roll call in progress.".to_string()),
            result.unwrap()
        );
    }

    #[test]
    fn handle_set_quiet() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = RollCall {
            quiet: true,
            ..create_call()
        };

        let responses = create_responses();

        scenario.expect(
            repo.update_quiet_call(2, arg!(true))
                .and_return(Ok(Some((call, responses)))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/shh".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("Ok fine, I\'ll be quiet. 🤐".to_string()),
            result.unwrap()
        );
    }

    #[test]
    fn handle_set_loud() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.update_quiet_call(2, arg!(false))
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/louder".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_update_quiet_no_in_progress() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        scenario.expect(repo.update_quiet_call(2, arg!(true)).and_return(Ok(None)));

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/shh".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("No roll call in progress.".to_string()),
            result.unwrap()
        );
    }

    #[test]
    fn handle_set_attendance_in() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.set_response_call(2, 1, "David", ANY)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "David".to_string(),
            command: "/in".to_string(),
            command_params: "will come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains("David (will come)"));
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_set_attendance_out() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.set_response_call(2, 1, "Daniel", ANY)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "Daniel".to_string(),
            command: "/out".to_string(),
            command_params: "won't come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains("Daniel (won't come)"));
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_set_attendance_maybe() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.set_response_call(2, 1, "Albert", ANY)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "Albert".to_string(),
            command: "/maybe".to_string(),
            command_params: "might come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains("David (will come)"));
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_set_attendance_no_in_progress() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        scenario.expect(
            repo.set_response_call(2, 1, "User 1", ANY)
                .and_return(Ok(None)),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/in".to_string(),
            command_params: "will come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("No roll call in progress.".to_string()),
            result.unwrap()
        );
    }

    #[test]
    fn handle_set_attendance_for_in() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.set_response_for_call(2, "David", ANY)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_in_for".to_string(),
            command_params: "David will come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains("David (will come)"));
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_set_attendance_for_out() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.set_response_for_call(2, "Daniel", ANY)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_out_for".to_string(),
            command_params: "Daniel won't come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains("Daniel (won't come)"));
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_set_attendance_for_maybe() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.set_response_for_call(2, "Albert", ANY)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_maybe_for".to_string(),
            command_params: "Albert might come".to_string(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains("Albert (might come)"));
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_set_attendance_for_empty() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/set_maybe_for".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("Please provide the person's name.".to_string()),
            result.unwrap()
        );
    }

    #[test]
    fn handle_get_all_attendances() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        let call = create_call();
        let responses = create_responses();

        scenario.expect(
            repo.get_call_with_responses_call(2)
                .and_return(Ok(Some((call, responses.clone())))),
        );

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/whos_in".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result: String = bot.handle(command).unwrap().unwrap();
        assert!(result.contains(&render_responses_full(&responses)));
    }

    #[test]
    fn handle_get_all_attendances_no_in_progress() {
        let scenario = Scenario::new();
        let repo = scenario.create_mock_for::<Repository>();

        scenario.expect(repo.get_call_with_responses_call(2).and_return(Ok(None)));

        let command = ChatCommand {
            chat_id: 2,
            user_id: 1,
            username: "User 1".to_string(),
            command: "/whos_in".to_string(),
            command_params: String::new(),
        };

        let bot = WhosInBot::new("", Box::new(repo));
        let result = bot.handle(command);
        assert_eq!(
            Some("No roll call in progress.".to_string()),
            result.unwrap()
        );
    }
}
