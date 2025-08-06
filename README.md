# Command-Pool: The Ultimate Concurrent Task Runner

[![crates.io](https://img.shields.io/crates/v/command-pool.svg)](https://crates.io/crates/command-pool)
[![docs.rs](https://docs.rs/command-pool/badge.svg)](https://docs.rs/command-pool)

`command-pool` is a powerful and intuitive command-line tool designed to execute a series of commands in parallel, giving you full control over concurrency and task management. Whether you're running benchmarks, processing data, or automating repetitive tasks, `command-pool` will significantly speed up your workflow.

## 🚀 Features

- **Concurrent Execution**: Run multiple instances of a command simultaneously.
- **Task Throttling**: Easily control the number of concurrent tasks with the `-c` or `--concurrency` flag.
- **Total Task Limit**: Specify the total number of tasks to run with the `-n` or `--total-tasks` flag.
- **Quiet Mode**: Suppress stdout from the executed commands to keep your output clean, showing only task start and end information.
- **Initial Launch Delay**: Stagger the launch of initial tasks to avoid overwhelming system resources.
- **Detailed Summary**: Get a comprehensive summary at the end, including total tasks, success/failure counts, success rate, and performance statistics (average, min, max duration).
- **Human-Readable Timestamps**: Durations are presented in a user-friendly format.

## 🛠️ Usage

The basic syntax for `command-pool` is:

```sh
command-pool [OPTIONS] -- <COMMAND> [ARGS]...
```

### Examples

**1. Basic Usage**

Run a simple shell command 10 times with a concurrency of 4:

```sh
command-pool -c 4 -n 10 -- bash -c "echo 'Hello from task' && sleep 1"
```

**2. Using a Script**

Execute a shell script multiple times. This is perfect for running tests or simulations.

```sh
command-pool -c 2 -n 5 -- bash demos/random_sleep.sh
```

**3. Quiet Mode**

Run tasks without seeing their stdout, focusing only on the `command-pool` summary:

```sh
command-pool -q -c 3 -n 10 -- bash demos/random_sleep.sh
```

## 📊 Example Output

```
Starting command-pool with:
  Concurrency: 2
  Total tasks: 5
  Command: bash demos/random_sleep.sh
  Quiet mode: false
  Initial launch delay: 100ms
----------------------------------------
[Task 1] Starting... (Running: 1)
[Task 2] Starting... (Running: 2)
[Task 1] Finished: Success (Exit Code: 0) (Running: 1)
[Task 1] Stdout:
Sleeping for 4 seconds...
Task finished after 4 seconds.

[Task 3] Starting... (Running: 2)
[Task 2] Finished: Success (Exit Code: 0) (Running: 1)
[Task 2] Stdout:
Sleeping for 6 seconds...
Task finished after 6 seconds.

...

----------------------------------------
All tasks completed.
Total: 5
Successful: 5
Failed: 0
Success Rate: 100.00%

Successful Tasks Statistics:
  Average Duration: 6.02s
  Min Duration: 3.01s
  Max Duration: 9.01s

Total command-pool execution time: 16.52s
```

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
