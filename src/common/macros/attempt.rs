macro_rules! attempt {
    ($action:block) => {{
        let process = || -> Result<(), Box<dyn std::error::Error>> { $action };
        process()
    }};
}

pub(crate) use attempt;