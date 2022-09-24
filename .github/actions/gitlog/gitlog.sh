#!/bin/bash

# This mangles git log entries for change lop purposes

output_file=""
crate=""
while true; do
  case $1 in
    "--output-file")
      shift
      output_file="$1"
      shift
    ;;
    "--crate")
      shift
      crate="$1"
      shift
    ;;
    *)
      break
    ;;
  esac
done

if [[ "$output_file" == "" ]]; then
  echo "Missing --output-file <file> option argument, define path to file or - for stdout" && exit 1
fi
if [[ "$crate" == "" ]]; then
  echo "Missing --crate <crate> option argument, need an explisit crate to get git log for" && exit 1
fi

from_commit=HEAD
last_release=$(git tag --sort=-committerdate | grep -E "$crate-[0-9]*\.[0-9]*\.[0-9]*" | head -1)
echo "Found tag: $last_release"
if [[ "$last_release" == "" ]]; then
  last_release=$(git tag --sort=-committerdate | head -1) # get last tag
  echo "Using latest tag: $last_release"
fi

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

function is_crate_related {
  commit="$1"
  changes="$(git diff --name-only "$commit"~ "$commit" | awk -F / '{print $1}' | xargs)"
  
  is_related=false
  if [[ "$changes" == *"$crate"* ]]; then
    is_related=true
  fi
  
  echo $is_related
}

log=""
for line in "${log_lines[@]}"; do
  commit=$(echo "$line" | awk -F ' ' '{print $1}')
  commit=${commit//[\(\)]/}
  
  if [[ $(is_crate_related "$commit") == true ]]; then
    log=$log"* $line\n"
  fi
done

if [[ "$output_file" != "" ]]; then
  if [[ "$output_file" == "-" ]]; then
    echo -e "$log"
  else
    echo -e "$log" > "$output_file"
  fi
fi

if [[ "$last_release" == "" ]]; then
  last_release=$(git rev-list --reverse HEAD | head -1)
fi
echo "::set-output name=last_release::$last_release"