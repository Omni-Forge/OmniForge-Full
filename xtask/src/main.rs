
use std::process::ExitStatus;

use clap::Parser;
use clap::Subcommand;
use futures::future::join_all;
#[derive(Parser)]
#[command(name = "Xtask")]
#[clap(rename_all = "snake_case")]
/// Cargo xtask runner made to easily deploy omni in a controlled testing enviornment with familar tools
struct Cli {
    #[arg(short,long,global = true)]
    verbose: bool,
    #[arg(short,long,global = true)]
    release: bool,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Clone, clap::ValueEnum)]
enum App {
    #[value(alias = "f")]
    Forge,
    #[value(alias = "d")]
    Director,
    #[value(alias = "a")]
    Agent,
    #[value(alias = "o")]
    Orchestrator,
    All,
    Docker,
}
#[derive(Clone,Subcommand)]
enum Commands {
    #[command(alias = "start")]
    Run {
        app: App,
    },
    Test,
    Build,
}
fn handle_commands(command: Commands) {
    println!("{}",command);
    match command {

        Commands::Run { app } => {
            match app {
                App::Forge => {
                    std::process::Command::new("cargo")
                        .arg("run")
                        .arg("--bin")
                        .arg("omniforge");
                    
                }
                App::Director => {
                    std::process::Command::new("cargo")
                        .arg("run")
                        .arg("--bin")
                        .arg("omnidirector");
                }
                App::Agent => {
                    std::process::Command::new("cargo")
                        .arg("run")
                        .arg("--bin")
                        .arg("omniagent");
                }
                App::All => {
                    futures::executor::block_on(run_all());
                }
                App::Docker => {
                    let output = std::process::Command::new("docker-compose")
                        .arg("up")
                        .arg("-d")
                        .current_dir("./docker")
                        .output()
                        
                        .expect("Failed to run docker stack");
                    if !output.status.success() {
                        println!("An error has occurred");
                        println!("{}",String::from_utf8_lossy(&output.stderr));
                    }
                    if output.stdout.is_empty() {
                        println!("Docker did not give any output");
                        return
                    }
                    println!("Command output: {:?}", String::from_utf8_lossy(&output.stdout))
                },
                App::Orchestrator => {
                    std::process::Command::new("cargo")
                    .arg("run")
                    .arg("--bin")
                    .arg("omniorchestrator")
                    .output().expect("Failed to run orchestrator");
                },
            }
        }
        Commands::Test => {
            println!("Testing");
        }
        Commands::Build => todo!(),
    }
}
fn main() {
    let cli = Cli::try_parse();
    match cli {
        Ok(cli) => handle_commands(cli.command),
        Err(e) => {
            e.print().expect("Failed to print error message")
        },
    }


}
async fn run_all() {
    let mut forge = std::process::Command::new("cargo");
    forge.arg("run").arg("--bin").arg("omni-forge").current_dir("OmniForge");
    
    let mut director = std::process::Command::new("cargo");
    director.arg("run").arg("--bin").arg("omni-director").current_dir("OmniDirector");
    
    let mut agent = std::process::Command::new("cargo");
    agent.arg("run").arg("--bin").arg("omni-agent").current_dir("OmniAgent");

    let mut orchestrator = std::process::Command::new("cargo");
    orchestrator.arg("run").arg("--bin").arg("omni-agent").current_dir("OmniOrchestrator");
    
    let commands = vec![forge, director, agent,orchestrator];

    let futures = commands.into_iter().map(|mut command| {
        async move {
            command.spawn().expect("Failed to start command")
        }
    });

    join_all(futures).await;
}