#!/bin/bash

# This mangles git log entries for change lop purposes

from_commit=HEAD
last_release=$(git tag --sort=-taggerdate | head -1) # get last tag

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

if [[ "$output_file" == "" ]]; then
  echo "Missing output file, did you forget to define --output-file <file>?" && exit 1
fi

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

for line in "${log_lines[@]}"; do
  log=$log"* $line"
  echo "$log" >> "$output_file"
done

cat < "$output_file"