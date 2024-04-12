use crate::errors::generator::generate_error_enum;

generate_error_enum!(LoginManagerInstanceError, {
   UnknownCurrentManager: "Default Manager is unsupported or it is not set.",
   UnsupportedManager: "Specific Manager is unsupported.",
   ManagerAlreadyDefault: "Specific manager is already current login manager.",
});