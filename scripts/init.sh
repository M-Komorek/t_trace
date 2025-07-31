# In scripts/init.sh

# Attempt to start the t_trace daemon in the background.
# The `t_trace daemon start` command is designed to be idempotent.
t_trace daemon start >/dev/null 2>&1 &

# Define the hook function to run before a command executes.
t_trace_preexec() {
  # Guard Clause: Do not track t_trace's own commands.
  if [[ "$BASH_COMMAND" == t_trace* ]]; then
    return
  fi

  # Run the client in the foreground. The '&' is removed.
  # This is extremely fast and guarantees the START message is sent before the command runs.
  t_trace client start "$BASHPID" "$BASH_COMMAND" >/dev/null 2>&1
}

# Define the hook function to run after a command has finished.
t_trace_precmd() {
  local exit_code=$?

  # Run the client in the foreground. The '&' is removed.
  # This guarantees the END message is sent before the next prompt is drawn.
  t_trace client end "$BASHPID" "$exit_code" >/dev/null 2>&1
}

# Register the functions with Bash's execution hooks.
trap 't_trace_preexec' DEBUG
export PROMPT_COMMAND="t_trace_precmd; $PROMPT_COMMAND"
