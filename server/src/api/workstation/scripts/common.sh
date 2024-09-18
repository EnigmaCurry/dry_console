########################################
## Common header for all bash scripts:
set -e
stderr(){ echo "$@" >/dev/stderr; }
error(){ stderr "Error: $@"; }
fault(){ test -n "$1" && error $1; stderr "Exiting."; exit 1; }
[ -z "$BASH_VERSION" ] || [ "${BASH_VERSINFO[0]}" -lt 5 ] && fault "Error: This script requires Bash v5 or higher!"
cancel(){ stderr "Canceled."; exit 2; }
exe() { (set -x; "$@"); }
print_array(){ printf '%s\n' "$@"; }
trim_trailing_whitespace() { sed -e 's/[[:space:]]*$//'; }
trim_leading_whitespace() { sed -e 's/^[[:space:]]*//'; }
trim_whitespace() { trim_leading_whitespace | trim_trailing_whitespace; }
expand_home_dir() { [[ $1 == "~/"* ]] && echo "${1/#\~/$HOME}" || echo "$1"; }
check_var(){ local __missing=false; local __vars="$@"; for __var in ${__vars}; do if [[ -z "${!__var}" ]]; then error "${__var} variable is missing."; __missing=true; fi; done; if [[ ${__missing} == true ]]; then fault; fi; }
check_num(){ local var=$1; check_var "$var"; [[ ${!var} =~ ^[0-9]+$ ]] || fault "${var} is not a number: '${!var}'"; }
debug_var(){ local var=$1; check_var "$var"; stderr "## DEBUG: ${var}=${!var}"; }
## End common header.
## Actual script starts below:
########################################
