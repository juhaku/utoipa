#!/bin/bash

# This mangles the releases for utoipa.


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

# log=""
if [[ $last_release != "" ]]; then
  # log=$(git log --pretty=format:%s--%D--%p "$from_commit"..."$last_release")
  mapfile -t log_lines < <(git log --pretty=format:%s--%D--%p --ancestry-path "$from_commit"..."$last_release")
  len=${#log_lines[@]}
  unset "log_lines[(($len - 1))]" # remove the last item since it is the actual tag
else
  # log=$(git log --pretty=format:%s--%D--%p "$from_commit")
  mapfile -t log_lines < <(git log --pretty=format:%s--%D--%p --ancestry-path "$from_commit")
fi

echo "log........"
version="0.0.0" # get this from somewhere from Cargo.toml?
echo "# Release v $version" # > _release_changes.md
for line in "${log_lines[@]}"; do
  # TODO should squash commit that are similar
  echo "* $line" # >> _release_changes.md
done

# TODO how to update the CHANGES.md??