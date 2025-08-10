t_trace daemon run >/dev/null 2>&1

# Define the hook function to run before a command executes.
t_trace_preexec() {
  # Guard Clause: Do not track t_trace's own commands.
  if [[ "$BASH_COMMAND" == t_trace* ]]; then
    return
  fi

  # This is extremely fast and guarantees the message command-start is sent before the command runs.
  t_trace daemon command-beings "$BASHPID" "$BASH_COMMAND" >/dev/null 2>&1
}

# Define the hook function to run after a command has finished.
t_trace_precmd() {
  local exit_code=$?

  # This guarantees the command-end message is sent before the next prompt is drawn.
  t_trace daemon command-end "$BASHPID" "$exit_code" >/dev/null 2>&1
}

# Register the functions with Bash's execution hooks.
trap 't_trace_preexec' DEBUG
export PROMPT_COMMAND="t_trace_precmd; $PROMPT_COMMAND"
