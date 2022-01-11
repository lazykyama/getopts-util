extern crate getopts;

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::process;

use getopts::HasArg;
use getopts::Occur;
use getopts::Options as GetOptOptions;

pub enum OptAction {
    StoreTrue,
    StoreFalse,
}

struct OptionDef {
    name: String,
    short_name: String,
    required: bool,
    multiple: bool,
    default: Option<String>,
    action: Option<OptAction>,
    help: String,
    uppercase_name: String,
}
impl OptionDef {
    fn new() -> Self {
        Self {
            name: String::from(""),
            short_name: String::from(""),
            required: false,
            multiple: false,
            default: None,
            action: None,
            help: String::from(""),
            uppercase_name: String::from(""),
        }
    }
}

pub struct Options {
    defined_names: Vec<String>,
    parsed_options: HashMap<String, Vec<String>>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            defined_names: Vec::new(),
            parsed_options: HashMap::new(),
        }
    }

    pub fn set_defined_names(&mut self, names: Vec<&String>) {
        for n in &names {
            self.defined_names.push(n.to_string());
        }
    }

    pub fn insert(&mut self, key: String, values: Vec<String>) -> Option<Vec<String>> {
        self.parsed_options.insert(key, values)
    }

    pub fn get(&self, key: &String) -> Option<&Vec<String>> {
        if self.parsed_options.contains_key(key) {
            self.parsed_options.get(key)
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: &String) -> bool {
        if self.parsed_options.contains_key(key) {
            true
        } else if self.defined_names.contains(key) {
            true
        } else {
            false
        }
    }

    pub fn defined_len(&self) -> usize {
        self.defined_names.len()
    }

    pub fn parsed_len(&self) -> usize {
        self.parsed_options.len()
    }
}

impl fmt::Debug for Options {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Options")
            .field("defined_names", &self.defined_names)
            .field("parsed_options", &self.parsed_options)
            .finish()
    }
}

pub struct OptionParser {
    opts: GetOptOptions,
    given_options: Vec<OptionDef>,
}

impl OptionParser {
    pub fn new() -> Self {
        Self {
            opts: GetOptOptions::new(),
            given_options: Vec::new(),
        }
    }

    fn show_usage(&self, program_name: &String) {
        let mut options_for_brief = "".to_string();
        for o in &self.given_options {
            let mut single_opt = format!(
                "--{}{}",
                o.name,
                if o.action.is_some() {
                    "".to_string()
                } else {
                    format!(" {}", o.uppercase_name)
                }
            );
            if o.multiple {
                single_opt = format!("{} {}...", single_opt, single_opt);
            }
            if !o.required {
                single_opt = format!("[{}]", single_opt);
            }
            single_opt = format!(" {}", single_opt);
            options_for_brief.push_str(&single_opt);
        }
        let brief = format!("Usage: {}{}", program_name, options_for_brief);
        println!("{}", self.opts.usage(&brief));
    }

    pub fn parse(&mut self) -> Options {
        let args = env::args().collect();
        self.parse_with_args(args)
    }

    fn parse_with_args(&mut self, args: Vec<String>) -> Options {
        // Always set help option if it's not specified by user.
        if !self.given_options.iter().any(|x| x.name == "help") {
            self.opts.opt(
                "h",
                "help",
                "show this help message and exit",
                "",
                HasArg::No,
                Occur::Optional,
            );
        }

        let matches = match self.opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                self.show_usage(&args[0]);
                if args[1..].iter().any(|x| x == "--help" || x == "-h") {
                    // If -h or --help exists in args,
                    // need to show help message and exit
                    // even though there is any other wrong option.
                    process::exit(0);
                } else {
                    // If simply wrong option exists without -h/--help,
                    // need to raise an error.
                    panic!("{}", f.to_string())
                }
            }
        };

        if matches.opt_present("h") {
            // When an user specifies -h or --help,
            // show usage help message and exit.
            self.show_usage(&args[0]);
            process::exit(0);
        }

        let mut options = Options::new();
        options.set_defined_names(self.given_options.iter().map(|x| &x.name).collect());
        for o in &self.given_options {
            if matches.opt_present(&o.name) {
                // option is given.
                if o.multiple {
                    // when this option accept multiple values.
                    if o.action.is_some() {
                        // for an option which accepts multiple values,
                        // this is not allowed to be used as a flag option.
                        panic!("{} must not be a flag option.", &o.name);
                    }

                    let opt_values = matches.opt_strs(&o.name);
                    if opt_values.len() > 0 {
                        // if any values are found, directly store them.
                        options.insert(o.name.clone(), opt_values);
                    } else {
                        // if no value is specified,
                        // check if the option is required or not.
                        if o.required {
                            panic!("{} is required option.", o.name);
                        }
                        // But, basically, when the code is reaching here,
                        // given option name and value may be not perfect.
                        panic!(
                            "{} must have a value, but only key like --{}",
                            o.name, o.name
                        );
                    }
                } else {
                    // when this option accept only single value.
                    let opt_value = matches.opt_str(&o.name);
                    if let Some(v) = opt_value {
                        // if a value is found, directly store it.
                        options.insert(o.name.clone(), vec![v]);
                    } else {
                        // if no value is specified,
                        // check if it's a flag option or an option with default value.
                        if let Some(v) = &o.action {
                            // flag option.
                            options.insert(
                                o.name.clone(),
                                vec![match v {
                                    OptAction::StoreTrue => "true".to_string(),
                                    OptAction::StoreFalse => "false".to_string(),
                                }],
                            );
                        } else {
                            // non flag option.
                            // But, basically, when the code is reaching here,
                            // given option name and value may be not perfect.
                            panic!(
                                "{} must have a value, but only key like --{}",
                                o.name, o.name
                            );
                        }
                    }
                }
            } else {
                // this option is not specified.
                // need to set default value.
                if let Some(v) = o.default.clone() {
                    options.insert(o.name.clone(), vec![v]);
                }
            }
        }

        options
    }

    pub fn add_option(
        &mut self,
        name: &str,
        short_name: &str,
        required: Option<bool>,
        multiple: Option<bool>,
        default: Option<&str>,
        action: Option<OptAction>,
        help: Option<&str>,
    ) {
        let mut option = OptionDef::new();

        option.name = name.to_string().clone();
        option.short_name = short_name.to_string().clone();
        option.required = if let Some(v) = required { v } else { false };
        option.multiple = if let Some(v) = multiple { v } else { false };
        option.default = if let Some(v) = default {
            Some(v.to_string().clone())
        } else {
            None
        };
        option.action = action;
        option.help = if let Some(v) = help {
            v.to_string().clone()
        } else {
            "".to_string()
        };

        option.uppercase_name = option.name.to_uppercase();

        // Set value to Options.
        let hasarg = if option.action.is_some() {
            // If `action` is specified,
            // this option is boolean flag,
            // and it must not have any values.
            HasArg::No
        } else {
            HasArg::Yes
        };
        let occur = if option.required {
            Occur::Req
        } else {
            if option.multiple {
                // NOTE: a combination of required and multiple will be checked later.
                Occur::Multi
            } else {
                Occur::Optional
            }
        };

        // Set all values.
        self.opts.opt(
            &option.short_name,
            &option.name,
            &option.help,
            &option.uppercase_name,
            hasarg,
            occur,
        );

        // Add this given option.
        self.given_options.push(option);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn get_program_name() -> String {
        env::current_exe()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    }

    fn setup_user_input_option(input_options: Vec<&str>) -> Vec<String> {
        let mut args: Vec<String> = vec![get_program_name()];
        let options: Vec<String> = input_options.iter().map(|x| x.to_string()).collect();
        args.extend(options);
        args
    }

    #[test]
    fn test_single_option() {
        let mut parser = OptionParser::new();
        let key = "input".to_string();
        parser.add_option(&key, "", None, None, None, None, None);

        let options = setup_user_input_option(vec!["--input", "INPUT_VALUE"]);
        let args = parser.parse_with_args(options);

        assert_eq!(args.defined_len(), 1);
        assert_eq!(args.parsed_len(), 1);
        assert!(args.contains_key(&key));
        let expected_value = vec!["INPUT_VALUE".to_string()];
        assert_eq!(args.get(&key), Some(&expected_value));
        assert_eq!(
            format!("{:?}", args),
            "Options { defined_names: [\"input\"], parsed_options: {\"input\": [\"INPUT_VALUE\"]} }",
        );
    }

    #[test]
    fn test_single_short_option() {
        let mut parser = OptionParser::new();
        let key = "input".to_string();
        let short_key = "i".to_string();
        parser.add_option(&key, &short_key, None, None, None, None, None);

        let options = setup_user_input_option(vec!["-i", "INPUT_VALUE"]);
        let args = parser.parse_with_args(options);

        assert_eq!(args.defined_len(), 1);
        assert_eq!(args.parsed_len(), 1);
        assert!(args.contains_key(&key));
        let expected_value = vec!["INPUT_VALUE".to_string()];
        assert_eq!(args.get(&key), Some(&expected_value));
        assert_eq!(
            format!("{:?}", args),
            "Options { defined_names: [\"input\"], parsed_options: {\"input\": [\"INPUT_VALUE\"]} }",
        );
    }

    #[test]
    fn test_single_flag_option() {
        let mut parser = OptionParser::new();
        let key = "verbose".to_string();
        parser.add_option(&key, "", None, None, None, Some(OptAction::StoreTrue), None);

        let options = setup_user_input_option(vec!["--verbose"]);
        let args = parser.parse_with_args(options);

        assert_eq!(args.defined_len(), 1);
        assert_eq!(args.parsed_len(), 1);
        assert!(args.contains_key(&key));
        let expected_value = vec!["true".to_string()];
        assert_eq!(args.get(&key), Some(&expected_value));
        assert_eq!(
            format!("{:?}", args),
            "Options { defined_names: [\"verbose\"], parsed_options: {\"verbose\": [\"true\"]} }",
        );
    }

    #[test]
    fn test_single_option_without_user_given_values() {
        let mut parser = OptionParser::new();
        let key = "input".to_string();
        parser.add_option(&key, "", None, None, None, None, None);
        let args = parser.parse();

        assert_eq!(args.defined_len(), 1);
        assert_eq!(args.parsed_len(), 0);
        assert!(args.contains_key(&key));
        assert_eq!(args.get(&key), None);
        assert_eq!(
            format!("{:?}", args),
            "Options { defined_names: [\"input\"], parsed_options: {} }",
        );
    }
}
