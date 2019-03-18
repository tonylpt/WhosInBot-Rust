use std::str::FromStr;

use regex::Regex;

use crate::models::{AttendanceStatus, ChatId, UserId};

use super::base_bot::ChatCommand;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    StartRollCall {
        chat_id: ChatId,
        title: String,
    },

    EndRollCall {
        chat_id: ChatId,
    },

    UpdateTitle {
        chat_id: ChatId,
        title: String,
    },

    UpdateQuiet {
        chat_id: ChatId,
        quiet: bool,
    },

    UpdateAttendanceSelf {
        chat_id: ChatId,
        user_id: UserId,
        username: String,
        status: AttendanceStatus,
        reason: String,
    },

    UpdateAttendanceFor {
        chat_id: ChatId,
        username: String,
        status: AttendanceStatus,
        reason: String,
    },

    GetAllAttendances {
        chat_id: ChatId,
    },

    ListAvailableCommands,
}

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum CommandParseError {
    #[fail(display = "missing title")]
    MissingTitle,

    #[fail(display = "missing username")]
    MissingUsername,

    #[fail(display = "Invalid command ({})", _0)]
    InvalidCommand(String),
}

impl Command {
    pub fn from_chat(chat_command: ChatCommand) -> Result<Self, CommandParseError> {
        use self::Command::*;
        use self::CommandParseError::*;
        use AttendanceStatus::*;

        let ChatCommand {
            chat_id,
            user_id,
            username,
            command,
            command_params,
            ..
        } = chat_command;

        match command.as_ref() {
            "/start_roll_call" => Ok(StartRollCall {
                chat_id,
                title: command_params,
            }),

            "/end_roll_call" => Ok(EndRollCall { chat_id }),

            "/set_title" => match command_params {
                ref title if title.is_empty() => Err(MissingTitle),
                title => Ok(UpdateTitle { chat_id, title }),
            },

            "/shh" => Ok(UpdateQuiet {
                chat_id,
                quiet: true,
            }),

            "/louder" => Ok(UpdateQuiet {
                chat_id,
                quiet: false,
            }),

            "/in" => Ok(UpdateAttendanceSelf {
                chat_id,
                user_id,
                username,
                status: In,
                reason: command_params,
            }),

            "/out" => Ok(UpdateAttendanceSelf {
                chat_id,
                user_id,
                username,
                status: Out,
                reason: command_params,
            }),

            "/maybe" => Ok(UpdateAttendanceSelf {
                chat_id,
                user_id,
                username,
                status: Maybe,
                reason: command_params,
            }),

            "/set_in_for" => command_params
                .parse()
                .map(|NameAndReason(username, reason)| UpdateAttendanceFor {
                    chat_id,
                    username,
                    status: In,
                    reason,
                }),

            "/set_out_for" => command_params
                .parse()
                .map(|NameAndReason(username, reason)| UpdateAttendanceFor {
                    chat_id,
                    username,
                    status: Out,
                    reason,
                }),

            "/set_maybe_for" => command_params
                .parse()
                .map(|NameAndReason(username, reason)| UpdateAttendanceFor {
                    chat_id,
                    username,
                    status: Maybe,
                    reason,
                }),

            "/whos_in" => Ok(GetAllAttendances { chat_id }),

            "/available_commands" | "/start" => Ok(ListAvailableCommands),

            unknown => Err(InvalidCommand(unknown.to_owned())),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct NameAndReason(String, String);

lazy_static! {
    static ref NAME_REASON_REGEX: Regex =
        Regex::new(r"^(\S+)\s*(.*)$").expect("Failed to create Regex");
}

impl FromStr for NameAndReason {
    type Err = CommandParseError;

    fn from_str(text: &str) -> Result<NameAndReason, CommandParseError> {
        if let Some(captures) = NAME_REASON_REGEX.captures(text) {
            let username = captures[1].to_owned();
            let reason = captures[2].to_owned();
            Ok(NameAndReason(username, reason))
        } else {
            Err(CommandParseError::MissingUsername)
        }
    }
}

#[cfg(test)]
mod tests {
    mod command_tests {
        use crate::models::AttendanceStatus::*;

        use super::super::{Command::*, *};

        #[test]
        fn test_from_start_roll_calll_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/start_roll_call".to_string(),
                command_params: "some title".to_string(),
            };

            let expected = Ok(StartRollCall {
                chat_id: 1,
                title: "some title".to_string(),
            });

            let actual = Command::from_chat(input);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_end_roll_call_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/end_roll_call".to_string(),
                command_params: "whatever".to_string(),
            };

            let expected = Ok(EndRollCall { chat_id: 1 });

            let actual = Command::from_chat(input);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_set_title_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/set_title".to_string(),
                command_params: "new title".to_string(),
            };

            let expected = Ok(UpdateTitle {
                chat_id: 1,
                title: "new title".to_string(),
            });

            let actual = Command::from_chat(input);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_shh_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/shh".to_string(),
                command_params: "whatever".to_string(),
            };

            let expected = Ok(UpdateQuiet {
                chat_id: 1,
                quiet: true,
            });

            let actual = Command::from_chat(input);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_louder_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/louder".to_string(),
                command_params: "whatever".to_string(),
            };

            let expected = Ok(UpdateQuiet {
                chat_id: 1,
                quiet: false,
            });

            let actual = Command::from_chat(input);
            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_self_attendance_command() {
            let input = vec!["/in", "/out", "/maybe"];
            let expected_status = vec![In, Out, Maybe];

            for (i, cmd) in input.into_iter().enumerate() {
                let input = ChatCommand {
                    chat_id: 1,
                    user_id: 2,
                    username: "Peter".to_string(),
                    command: cmd.to_string(),
                    command_params: "my reason".to_string(),
                };

                let expected = Ok(UpdateAttendanceSelf {
                    chat_id: 1,
                    user_id: 2,
                    username: "Peter".to_string(),
                    status: expected_status[i],
                    reason: "my reason".to_string(),
                });

                let actual = Command::from_chat(input);
                assert_eq!(expected, actual);
            }
        }

        #[test]
        fn test_from_attendance_for_command() {
            let input = vec!["/set_in_for", "/set_out_for", "/set_maybe_for"];
            let expected_status = vec![In, Out, Maybe];

            for (i, cmd) in input.into_iter().enumerate() {
                let input = ChatCommand {
                    chat_id: 1,
                    user_id: 2,
                    username: "User 1".to_string(),
                    command: cmd.to_string(),
                    command_params: "Peter some reason".to_string(),
                };

                let expected = Ok(UpdateAttendanceFor {
                    chat_id: 1,
                    username: "Peter".to_string(),
                    status: expected_status[i],
                    reason: "some reason".to_string(),
                });

                let actual = Command::from_chat(input);
                assert_eq!(expected, actual);
            }
        }

        #[test]
        fn test_from_whosin_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/whos_in".to_string(),
                command_params: "whatever".to_string(),
            };

            let expected = Ok(GetAllAttendances { chat_id: 1 });
            let actual = Command::from_chat(input);

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_start_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/start".to_string(),
                command_params: "whatever".to_string(),
            };

            let expected = Ok(ListAvailableCommands);
            let actual = Command::from_chat(input);

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_from_available_command() {
            let input = ChatCommand {
                chat_id: 1,
                user_id: 2,
                username: "Peter".to_string(),
                command: "/available_commands".to_string(),
                command_params: "whatever".to_string(),
            };

            let expected = Ok(ListAvailableCommands);
            let actual = Command::from_chat(input);

            assert_eq!(expected, actual);
        }
    }

    mod name_and_reason_tests {
        use super::super::*;

        #[test]
        fn test_parse_from_valid_string() {
            let input = "Peter will come";
            let expected = Ok(NameAndReason("Peter".to_string(), "will come".to_string()));
            let actual = input.parse::<NameAndReason>();

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_parse_from_valid_string_without_reason() {
            let input = "Peter  ";
            let expected = Ok(NameAndReason("Peter".to_owned(), "".to_owned()));
            let actual = input.parse::<NameAndReason>();

            assert_eq!(expected, actual);
        }

        #[test]
        fn test_parse_from_invalid_string() {
            let input = "   ";
            let expected = Err(CommandParseError::MissingUsername);
            let actual = input.parse::<NameAndReason>();

            assert_eq!(expected, actual);
        }
    }
}
