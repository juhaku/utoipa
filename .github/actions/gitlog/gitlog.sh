#!/bin/bash

# This mangles git log entries for change lop purposes

from_commit=HEAD
last_release=$(git tag --sort=-taggerdate | head -1) # get last tag
# tags=()
# tags=($(git tag | xargs))
# mapfile -t tags < <(git tag)

# for tag in "${tags[@]}"; do
#   echo "tag $tag"
# done

# function has_tags {
#   [[ ${#tags[@]} -gt 0 ]] && echo true || echo false
# }

# echo ${#tags[@]} -gt 0
# echo $(has_tags) "--" ${#tags[@]}

output_file=""
while true; do
  case $1 in
    "--output-file")
      shift
      output_file="$1"
      shift
    ;;
    "")
      break
    ;;
    *)
      break
    ;;
  esac
done

commit_range=""
if [[ $last_release != "" ]]; then
  commit_range="$from_commit...$last_release"
  len=${#log_lines[@]}
  unset "log_lines[(($len - 1))]" # remove the last item since it is the actual tag
else
  commit_range="$from_commit"
fi

ancestry_path=""
if [[ "$last_release" != "" ]]; then
  ancestry_path="--ancestry-path"
fi

mapfile -t log_lines < <(git log --pretty=format:'(%p) %s' $ancestry_path $commit_range)

log=""
for line in "${log_lines[@]}"; do
  log=$log"* $line\n"
  # echo "* $line" # >> _release_changes.md
done
echo "::set-output name=commits::$(echo -e "$log")"
# echo -e "$log"
if [[ "$output_file" != "" ]]; then
  echo -e "$log" > "$output_file"
fi
# TODO how to update the CHANGES.md??