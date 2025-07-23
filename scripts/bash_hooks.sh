__t_trace_preexec() {
  local this_command="$BASH_COMMAND"

  # If the command is empty, part of the prompt machinery, or our own tool, ignore it.
  # The regex checks if $this_command is a standalone command within PROMPT_COMMAND.
  if [[ -z "$this_command" || "$PROMPT_COMMAND" =~ (^|;)${this_command}($|;) || "$this_command" =~ ^t_trace ]]; then
    return
  fi

  command t_trace start --command "$this_command"
}

__t_trace_precmd() {
  command t_trace end
}

trap - DEBUG
trap '__t_trace_preexec' DEBUG

if [[ -z "$PROMPT_COMMAND" ]]; then
  PROMPT_COMMAND="__t_trace_precmd"
elif [[ ! "$PROMPT_COMMAND" =~ __t_trace_precmd ]]; then
  PROMPT_COMMAND="__t_trace_precmd;$PROMPT_COMMAND"
fi
