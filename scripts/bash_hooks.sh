__t_trace_preexec() {
  if [[ -z "$BASH_COMMAND" || "$BASH_COMMAND" == "$PROMPT_COMMAND" || "$BASH_COMMAND" =~ ^t_trace ]]; then
    return
  fi

  command t_trace start --command "$BASH_COMMAND"
}

__t_trace_precmd() {
  command t_trace end
}

trap '__t_trace_preexec' DEBUG

if [[ -z "$PROMPT_COMMAND" ]]; then
  PROMPT_COMMAND="__t_trace_precmd"
elif [[ ! "$PROMPT_COMMAND" =~ __t_trace_precmd ]]; then
  PROMPT_COMMAND="__t_trace_precmd;$PROMPT_COMMAND"
fi
