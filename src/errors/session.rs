use crate::errors::generator::generate_error_enum;

generate_error_enum!(SessionInstanceError,
    {
        DefaultSessionNotSet: "Default session is not set or no session is specified.",
        SessionNotFoundInConfig: "Specific session is not found in config.",
        SessionNotFoundInSystem: "Specific session is not found in system.",
        UnknownProtocol: "Session Protocol is unknown or not supported.",
        LogoutCommandNotSet: "Logout command is not set",
        SessionExists: "Specific session already exists",
    }
);