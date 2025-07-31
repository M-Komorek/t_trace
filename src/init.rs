use crate::cli::Shell;

const BASH_SCRIPT: &str = include_str!("../scripts/init.sh");

pub fn print_script(shell: Shell) {
    match shell {
        Shell::Bash => {
            println!("{}", BASH_SCRIPT);
        }
    }
}
