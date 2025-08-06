use argh::FromArgs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::{self, Duration, Instant};

#[derive(FromArgs, Debug)]
/// a command-pool to run multiple commands in parallel.
struct Args {
  /// number of concurrent tasks
  #[argh(option, short = 'c', default = "1")]
  concurrency: usize,

  /// total number of tasks to execute
  #[argh(option, short = 'n')]
  total_tasks: usize,

  /// hide some-command specific stdout output, only show task start/end info
  #[argh(switch, short = 'q')]
  quiet: bool,

  /// delay between initial task launches in milliseconds
  #[argh(option, short = 'd', default = "100")]
  delay: u64,

  /// the command and its arguments to execute
  #[argh(positional, greedy)]
  command: Vec<String>,
}

fn format_duration_custom(duration: Duration) -> String {
  let secs = duration.as_secs();
  if secs >= 60 {
    humantime::format_duration(Duration::from_secs(secs)).to_string()
  } else {
    format!("{:.2}s", duration.as_secs_f64())
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Args = argh::from_env();

  if args.command.is_empty() {
    eprintln!("Error: No command provided to execute.");
    std::process::exit(1);
  }

  let command_str = args.command[0].clone();
  let command_args = args.command[1..].to_vec();

  println!("Starting command-pool with:");
  println!("  Concurrency: {}", args.concurrency);
  println!("  Total tasks: {}", args.total_tasks);
  println!("  Command: {} {}", command_str, command_args.join(" "));
  println!("  Quiet mode: {}", args.quiet);
  println!("  Initial launch delay: {}ms", args.delay);
  println!("----------------------------------------");

  let start_time = Instant::now(); // Overall start time

  let mut join_set = JoinSet::new();
  let completed_tasks = Arc::new(AtomicUsize::new(0));
  let successful_tasks = Arc::new(AtomicUsize::new(0));
  let failed_tasks = Arc::new(AtomicUsize::new(0));
  let running_tasks = Arc::new(AtomicUsize::new(0));
  let successful_durations = Arc::new(Mutex::new(Vec::<Duration>::new())); // New: Store successful task durations
  let failed_durations = Arc::new(Mutex::new(Vec::<Duration>::new())); // New: Store failed task durations

  let mut task_id_counter = 0;

  // Spawn initial tasks up to concurrency limit
  for i in 0..args.concurrency.min(args.total_tasks) {
    task_id_counter += 1;
    let task_id = task_id_counter;
    let cmd_str_clone = command_str.clone();
    let cmd_args_clone = command_args.clone();
    let quiet_clone = args.quiet;
    let completed_tasks_clone = Arc::clone(&completed_tasks);
    let successful_tasks_clone = Arc::clone(&successful_tasks);
    let failed_tasks_clone = Arc::clone(&failed_tasks);
    let running_tasks_clone = Arc::clone(&running_tasks);
    let successful_durations_clone = Arc::clone(&successful_durations);
    let failed_durations_clone = Arc::clone(&failed_durations);

    join_set.spawn(async move {
      running_tasks_clone.fetch_add(1, Ordering::SeqCst);
      println!(
        "[Task {}] Starting... (Running: {})",
        task_id,
        running_tasks_clone.load(Ordering::SeqCst)
      );
      let mut cmd = Command::new(&cmd_str_clone);
      cmd.args(&cmd_args_clone);

      let task_start_time = Instant::now(); // Task start time
      let output_result = cmd.output().await;
      let task_duration = task_start_time.elapsed(); // Task duration

      let (result_msg, stdout_output, stderr_output) = match output_result {
        Ok(output) => {
          let stdout = String::from_utf8_lossy(&output.stdout).to_string();
          let stderr = String::from_utf8_lossy(&output.stderr).to_string();
          if output.status.success() {
            successful_tasks_clone.fetch_add(1, Ordering::SeqCst);
            successful_durations_clone.lock().unwrap().push(task_duration); // Store duration
            (
              format!("Success (Exit Code: {})", output.status.code().unwrap_or_default()),
              stdout,
              stderr,
            )
          } else {
            failed_tasks_clone.fetch_add(1, Ordering::SeqCst);
            failed_durations_clone.lock().unwrap().push(task_duration); // Store duration
            (
              format!("Failed (Exit Code: {})", output.status.code().unwrap_or_default()),
              stdout,
              stderr,
            )
          }
        }
        Err(e) => {
          failed_tasks_clone.fetch_add(1, Ordering::SeqCst);
          failed_durations_clone.lock().unwrap().push(task_duration); // Store duration
          (format!("Error: {e}"), String::new(), String::new())
        }
      };

      completed_tasks_clone.fetch_add(1, Ordering::SeqCst);
      running_tasks_clone.fetch_sub(1, Ordering::SeqCst);
      println!(
        "[Task {}] Finished: {} (Running: {})",
        task_id,
        result_msg,
        running_tasks_clone.load(Ordering::SeqCst)
      );
      if !quiet_clone && !stdout_output.is_empty() {
        println!(
          "[Task {task_id}] Stdout:
{stdout_output}"
        );
      }
      if !stderr_output.is_empty() {
        eprintln!(
          "[Task {task_id}] Stderr:
{stderr_output}"
        );
      }
      task_id
    });

    // Apply delay only for initial launches, and not after the last initial task
    if args.delay > 0 && i < args.concurrency.min(args.total_tasks) - 1 {
      time::sleep(Duration::from_millis(args.delay)).await;
    }
  }

  // Continuously spawn new tasks as old ones complete, until total_tasks is reached
  while let Some(res) = join_set.join_next().await {
    let _finished_task_id = res?; // Handle potential panics in spawned tasks

    if task_id_counter < args.total_tasks {
      task_id_counter += 1;
      let task_id = task_id_counter;
      let cmd_str_clone = command_str.clone();
      let cmd_args_clone = command_args.clone();
      let quiet_clone = args.quiet;
      let completed_tasks_clone = Arc::clone(&completed_tasks);
      let successful_tasks_clone = Arc::clone(&successful_tasks);
      let failed_tasks_clone = Arc::clone(&failed_tasks);
      let running_tasks_clone = Arc::clone(&running_tasks);
      let successful_durations_clone = Arc::clone(&successful_durations);
      let failed_durations_clone = Arc::clone(&failed_durations);

      join_set.spawn(async move {
        running_tasks_clone.fetch_add(1, Ordering::SeqCst);
        println!(
          "[Task {}] Starting... (Running: {})",
          task_id,
          running_tasks_clone.load(Ordering::SeqCst)
        );
        let mut cmd = Command::new(&cmd_str_clone);
        cmd.args(&cmd_args_clone);

        let task_start_time = Instant::now(); // Task start time
        let output_result = cmd.output().await;
        let task_duration = task_start_time.elapsed(); // Task duration

        let (result_msg, stdout_output, stderr_output) = match output_result {
          Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
              successful_tasks_clone.fetch_add(1, Ordering::SeqCst);
              successful_durations_clone.lock().unwrap().push(task_duration); // Store duration
              (
                format!("Success (Exit Code: {})", output.status.code().unwrap_or_default()),
                stdout,
                stderr,
              )
            } else {
              failed_tasks_clone.fetch_add(1, Ordering::SeqCst);
              failed_durations_clone.lock().unwrap().push(task_duration); // Store duration
              (
                format!("Failed (Exit Code: {})", output.status.code().unwrap_or_default()),
                stdout,
                stderr,
              )
            }
          }
          Err(e) => {
            failed_tasks_clone.fetch_add(1, Ordering::SeqCst);
            failed_durations_clone.lock().unwrap().push(task_duration); // Store duration
            (format!("Error: {e}"), String::new(), String::new())
          }
        };

        completed_tasks_clone.fetch_add(1, Ordering::SeqCst);
        running_tasks_clone.fetch_sub(1, Ordering::SeqCst);
        println!(
          "[Task {}] Finished: {} (Running: {})",
          task_id,
          result_msg,
          running_tasks_clone.load(Ordering::SeqCst)
        );
        if !quiet_clone && !stdout_output.is_empty() {
          println!(
            "[Task {task_id}] Stdout:
{stdout_output}"
          );
        }
        if !stderr_output.is_empty() {
          eprintln!(
            "[Task {task_id}] Stderr:
{stderr_output}"
          );
        }
        task_id
      });
    }

    if completed_tasks.load(Ordering::SeqCst) == args.total_tasks {
      break;
    }
  }

  let total_duration = start_time.elapsed(); // Overall end time

  println!("----------------------------------------");
  println!("All tasks completed.");
  println!("Total: {}", completed_tasks.load(Ordering::SeqCst));
  println!("Successful: {}", successful_tasks.load(Ordering::SeqCst));
  println!("Failed: {}", failed_tasks.load(Ordering::SeqCst));

  let success_rate = if args.total_tasks > 0 {
    (successful_tasks.load(Ordering::SeqCst) as f64 / args.total_tasks as f64) * 100.0
  } else {
    0.0
  };
  println!("Success Rate: {success_rate:.2}%");

  // Report for successful tasks
  let successful_durations_locked = successful_durations.lock().unwrap();
  if !successful_durations_locked.is_empty() {
    let sum_duration: Duration = successful_durations_locked.iter().sum();
    let avg_duration = sum_duration / successful_durations_locked.len() as u32;
    let min_duration = successful_durations_locked.iter().min().unwrap();
    let max_duration = successful_durations_locked.iter().max().unwrap();
    println!("\nSuccessful Tasks Statistics:");
    println!("  Average Duration: {}", format_duration_custom(avg_duration));
    println!("  Min Duration: {}", format_duration_custom(*min_duration));
    println!("  Max Duration: {}", format_duration_custom(*max_duration));
  }

  // Report for failed tasks
  let failed_durations_locked = failed_durations.lock().unwrap();
  if !failed_durations_locked.is_empty() {
    let sum_duration: Duration = failed_durations_locked.iter().sum();
    let avg_duration = sum_duration / failed_durations_locked.len() as u32;
    let min_duration = failed_durations_locked.iter().min().unwrap();
    let max_duration = failed_durations_locked.iter().max().unwrap();
    println!("\nFailed Tasks Statistics:");
    println!("  Average Duration: {}", format_duration_custom(avg_duration));
    println!("  Min Duration: {}", format_duration_custom(*min_duration));
    println!("  Max Duration: {}", format_duration_custom(*max_duration));
  }

  println!("\nTotal command-pool execution time: {}", format_duration_custom(total_duration));

  Ok(())
}
