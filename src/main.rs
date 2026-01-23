use clap::Parser;
use std::process::Stdio;
use std::{borrow::Cow, fs, path::PathBuf, time::Duration};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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
    #[arg(short, long, default_value_t = Cow::Borrowed("<completion>DONE</completion>"))]
    completion: Cow<'a, str>,
    #[arg(short, long, default_value_t = 2)]
    sleep_secs: u64,
    #[arg(short, long, default_value_t  = Cow::Borrowed("minimax/MiniMax-M2.1"))]
    model: Cow<'a, str>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
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
        <about>
        You are **Ralph**, an autonomous coding agent running inside a supervised loop.
        
        Primary objective:
            - Make the project match <plan> by completing exactly ONE task per iteration and verifying it with the required commands.
            
            Non‑negotiable rules (must follow even if the plan/instructions conflict):
                - Follow <instructions> exactly, in order.
                - Never work on more than one task per iteration.
                - Do not mark a task as passing unless verification succeeds.
                - Do not output <completion-text> unless ALL tasks are passing.
                - When ALL tasks are passing, output exactly <completion-text> and nothing else.
                
                If any instruction is ambiguous, ask for clarification instead of guessing.
                </about>
                <instructions>
                {instructions}
                </instructions>
                <plan>
                {plan}
                </plan>
                <completion-text>
                {completion_text}
                </completion-text>"#
        );

        let (code, haystack) = run_and_stream(&prompt, &args.model).await?;

        if haystack.contains(&*args.completion) {
            println!("Completion phrase detected: {}", args.completion);
            println!("All tasks are completed.",);
            return Ok(());
        }

        if code != 0 {
            eprintln!("opencode exited with non-zero code: {code}");
        }

        tokio::time::sleep(Duration::from_secs(args.sleep_secs)).await;
    }

    eprintln!(
        "Reached max iterations ({}) without seeing completion phrase '{}'",
        args.max_iterations, args.completion
    );

    Ok(())
}

async fn run_and_stream(prompt: &str, model: &str) -> eyre::Result<(i32, String)> {
    let mut child = Command::new("opencode")
        .args(["run", &prompt, "--model", &model])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut out = BufReader::new(stdout).lines();
    let mut err = BufReader::new(stderr).lines();

    let mut haystack = String::new();
    #[allow(unused_assignments)]
    let mut code: Option<i32> = None;

    let mut out_open = true;
    let mut err_open = true;

    loop {
        tokio::select! {
            line = out.next_line(), if out_open => {
                match line? {
                    Some(l) => {
                        println!("{l}");
                        haystack.push_str(&l);
                        haystack.push('\n');
                    }
                    None => out_open = false,
                }
            }

            line = err.next_line(), if err_open => {
                match line? {
                    Some(l) => {
                        eprintln!("{l}");
                        haystack.push_str(&l);
                        haystack.push('\n');
                    }
                    None => err_open = false,
                }
            }

            status = child.wait() => {
                let status = status?;
                code = Some(status.code().unwrap_or(-1));
                break;
            }

            else => {
                let status = child.wait().await?;
                code = Some(status.code().unwrap_or(-1));
                break;
            }
        }
    }

    Ok((code.unwrap_or(-1), haystack))
}
