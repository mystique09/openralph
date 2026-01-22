use clap::Parser;
use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = Some("A Ralph Wiggum implementation with Rust and Opencode"))]
struct Args<'a>
where
    'a: 'static,
{
    #[arg(short, long)]
    prompt: PathBuf,
    #[arg(short = 'n', long, default_value_t = 10)]
    max_iterations: u16,
    #[arg(short, long, default_value_t = Cow::Borrowed("DONE"))]
    completion: Cow<'a, str>,
    #[arg(short, long, default_value_t = 2)]
    sleep_secs: u64,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let instructions = include_str!("../instructions.md");
    let completion_text = &args.completion;

    println!("{:?}", args);

    for i in 1..args.max_iterations {
        eprintln!("--- iteration {i}/{} ---", args.max_iterations,);

        let plan = fs::read_to_string(&args.prompt)?;

        let prompt = format!(
            r#"
<instructions>
    {instructions}
</instructions>
<plan>
    {plan}
</plan>
<completion-text>{completion_text}</completion-text>"#
        );

        let child = Command::new("opencode")
            .args(["run", &prompt])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let out = child.wait_with_output()?;
        let code = out.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let haystack = format!("{stdout}\n{stderr}");

        if !stdout.is_empty() {
            print!("{stdout}");
        }

        if !stderr.is_empty() {
            eprint!("{stderr}");
        }

        if haystack.contains(&*args.completion) {
            println!("Completion phrase detected: {}", args.completion);
            println!("All tasks are completed.",);
            return Ok(());
        }

        if code != 0 {
            eprintln!("opencode exited with non-zero code: {code}");
        }

        thread::sleep(Duration::from_secs(args.sleep_secs));
    }

    eprintln!(
        "Reached max iterations ({}) without seeing completion phrase '{}'",
        args.max_iterations, args.completion
    );

    Ok(())
}
