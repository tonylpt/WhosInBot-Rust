use failure::SyncFailure;
use futures::Stream;
use regex::Regex;
use slog_scope;
use telegram_bot::{self, *};
use tokio_core::reactor::Core;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatCommand {
    pub chat_id: i64,
    pub user_id: i64,
    pub username: String,
    pub command: String,
    pub command_params: String,
}

pub type BotResult = Result<(), failure::Error>;

pub fn run<F>(token: &str, handler: F) -> BotResult
where
    F: Fn(ChatCommand) -> Result<Option<String>, failure::Error>,
{
    let mut core = Core::new().map_err(SyncFailure::new)?;
    let api = Api::configure(token)
        .build(core.handle())
        .map_err(SyncFailure::new)?;

    let logger = slog_scope::logger();

    info!("Waiting for messages...");
    let future = api.stream().for_each(|update| {
        if let UpdateKind::Message(message) = update.kind {
            if let Some(reply) = handle_message(&handler, &message, &logger) {
                api.spawn(message.chat.text(reply))
            }
        }
        Ok(())
    });

    core.run(future).map_err(|e| SyncFailure::new(e).into())
}

fn handle_message<F>(handler: &F, message: &Message, logger: &slog::Logger) -> Option<String>
where
    F: Fn(ChatCommand) -> Result<Option<String>, failure::Error>,
{
    if let Some(command) = parse_message(message) {
        let chat_id = command.chat_id;
        let user_id = command.user_id;

        let sentry_scope = |scope: &mut sentry::Scope| {
            scope.set_tag("chat_id", chat_id);
            scope.set_tag("user_id", user_id);
        };

        let logger_scope = &logger.new(o!(
            "chat_id" => chat_id,
            "user_id" => user_id,
        ));

        let handle = || match handler(command) {
            Ok(Some(reply)) => Some(reply),
            Ok(None) => None,
            Err(err) => {
                error!("An error has occurred: {}", err; "details" => format!("{:?}", err));
                sentry::integrations::failure::capture_error(&err);
                Some("An error has occurred.".to_string())
            }
        };

        return sentry::with_scope(sentry_scope, || slog_scope::scope(logger_scope, handle));
    }

    None
}

lazy_static! {
    static ref COMMAND_REGEX: Regex =
        Regex::new(r"^(/[^@[:space:]]+)(@\S*)?\s*(.*)$").expect("Failed to create Regex");
}

fn parse_message(message: &Message) -> Option<ChatCommand> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        if let Some(captures) = COMMAND_REGEX.captures(data) {
            let command = captures[1].to_owned();
            let command_params = captures[3].trim_end().to_owned();

            return Some(ChatCommand {
                chat_id: message.chat.id().into(),
                user_id: message.from.id.into(),
                username: message.from.first_name.clone(),
                command,
                command_params,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::util::testutil::*;

    use super::*;

    fn build_message(text: &str) -> Message {
        Message {
            id: MessageId::from(1234),
            from: User {
                id: UserId::from(12345),
                first_name: "User 1".to_string(),
                last_name: None,
                username: Some("user_1".to_string()),
            },
            date: 12345,
            chat: MessageChat::Group(Group {
                id: GroupId::from(123),
                title: "group title".to_string(),
                all_members_are_administrators: false,
            }),
            forward: None,
            reply_to_message: None,
            edit_date: None,
            kind: MessageKind::Text {
                data: text.to_string(),
                entities: vec![],
            },
        }
    }

    mod parse_message_tests {
        use super::*;

        #[test]
        fn test_parse_valid_command() {
            let message = build_message("/some_command command params ");
            let actual = parse_message(&message);
            let expected = Some(ChatCommand {
                chat_id: 123,
                user_id: 12345,
                username: "User 1".to_string(),
                command: "/some_command".to_string(),
                command_params: "command params".to_string(),
            });

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_parse_valid_command_with_mention() {
            let message = build_message("/some_command@bot command params ");
            let actual = parse_message(&message);
            let expected = Some(ChatCommand {
                chat_id: 123,
                user_id: 12345,
                username: "User 1".to_string(),
                command: "/some_command".to_string(),
                command_params: "command params".to_string(),
            });

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_parse_invalid_commands() {
            assert_eq!(None, parse_message(&build_message("")));
            assert_eq!(None, parse_message(&build_message("  ")));
            assert_eq!(None, parse_message(&build_message("invalid")));
            assert_eq!(
                None,
                parse_message(&build_message("invalid /invalid command params"))
            );
        }
    }

    mod handle_message_tests {
        use super::*;

        #[test]
        fn test_handle_message_with_success_result() {
            let message = build_message("/command params");
            let handler = |command: ChatCommand| {
                Ok(Some(format!(
                    "response to {} {} from {} in chat {}",
                    command.command, command.command_params, command.user_id, command.chat_id
                )))
            };

            let result = with_test_logger(|logger| handle_message(&handler, &message, logger));

            assert_eq!(
                Some("response to /command params from 12345 in chat 123".to_string()),
                result
            );
        }

        #[test]
        fn test_handle_message_with_empty_result() {
            let message = build_message("/command params");
            let handler = |_: ChatCommand| Ok(None);

            let result = with_test_logger(|logger| handle_message(&handler, &message, logger));

            assert_eq!(None, result);
        }

        #[test]
        fn test_handle_message_with_error_result() {
            let message = build_message("/command params");
            let handler = |_: ChatCommand| bail!("mock error");

            let result = with_test_logger(|logger| handle_message(&handler, &message, logger));

            assert_eq!(Some("An error has occurred.".to_string()), result);
        }
    }
}
