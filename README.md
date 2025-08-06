# command-pool

`command-pool` is a command-line tool for running a command multiple times in parallel.

## Features

- **Concurrent Execution**: Run multiple instances of a command simultaneously.
- **Concurrency Control**: Control the number of concurrent tasks with the `-c` or `--concurrency` flag.
- **Task Limit**: Specify the total number of tasks to run with the `-n` or `--total-tasks` flag.
- **Task Timeout**: Set a timeout for each task in seconds with the `--timeout` option.
- **Stop on Failure**: Stop spawning new tasks if one fails with the `--stop-on-fail` flag.
- **Quiet Mode**: Suppress stdout from the executed commands.
- **Launch Delay**: Configure a delay between the initial task launches.
- **Summary Report**: A summary is provided after execution with statistics on task completion and duration.

## Usage

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

Execute a shell script multiple times.

```sh
command-pool -c 2 -n 5 -- bash demos/random_sleep.sh
```

**3. With Timeout**

Run tasks with a 2-second timeout.

```sh
command-pool -c 3 -n 10 --timeout 2 -- bash demos/random_sleep.sh
```

## License

This project is licensed under the MIT License.
