#!/bin/bash

# This mangles git log entries for change lop purposes

from_commit=HEAD
last_release=$(git tag | sort -r | head -1) # get last tag

output_file=""
while true; do
  case $1 in
    "--output-file")
      shift
      output_file="$1"
      shift
    ;;
    *)
      break
    ;;
  esac
done

commit_range=""
if [[ $last_release != "" ]]; then
  commit_range="$from_commit...$last_release"
else
  commit_range="$from_commit"
fi

ancestry_path=""
if [[ "$last_release" != "" ]]; then
  ancestry_path="--ancestry-path"
fi

mapfile -t log_lines < <(git log --pretty=format:'(%h) %s' $ancestry_path $commit_range)

log=""
for line in "${log_lines[@]}"; do
  log=$log"* $line\n"
done
if [[ "$output_file" != "" ]]; then
  echo -e "$log" > "$output_file"
fi
