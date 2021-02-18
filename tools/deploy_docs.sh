#!/bin/bash

set -e
# set -ex

function print_error() {
    echo -e "\e[31mERROR: ${1}\e[m"
}

function print_info() {
    echo -e "\e[36mINFO: ${1}\e[m"
}

function skip() {
    print_info "No changes detected, skipping deployment"
    exit 0
}

# check values
if [ -n "${EXTERNAL_REPOSITORY}" ]; then
    PUBLISH_REPOSITORY=${EXTERNAL_REPOSITORY}
else
    PUBLISH_REPOSITORY=${GITHUB_REPOSITORY}
fi
print_info "Deploy to ${PUBLISH_REPOSITORY}"

remote_repo="git@github.com:${PUBLISH_REPOSITORY}.git"

if [ -z "${PUBLISH_BRANCH}" ]; then
    print_error "not found PUBLISH_BRANCH"
    exit 1
fi

if [ -z "${PUBLISH_DIR}" ]; then
    print_error "not found PUBLISH_DIR"
    exit 1
fi

remote_branch="${PUBLISH_BRANCH}"

local_dir="${HOME}/ghpages_${RANDOM}"
if git clone --depth=1 --single-branch --branch "${remote_branch}" "${remote_repo}" "${local_dir}"; then
    cd "${local_dir}"

    if [[ ${INPUT_KEEPFILES} == "true" ]]; then
        print_info "Keeping existing files: ${INPUT_KEEPFILES}"
    else
        git rm -r --ignore-unmatch '*'
    fi

    find "${GITHUB_WORKSPACE}/${PUBLISH_DIR}" -maxdepth 1 | \
        tail -n +2 | \
        xargs -I % cp -rf % "${local_dir}/"
else
    cd "${PUBLISH_DIR}"
    git init
    git checkout --orphan "${remote_branch}"
fi

# push to publishing branch
git config user.name "${GITHUB_ACTOR}"
git config user.email "${GITHUB_ACTOR}@users.noreply.github.com"
git remote rm origin || true
git remote add origin "${remote_repo}"
git add --all

print_info "Allowing empty commits: ${INPUT_EMPTYCOMMITS}"
COMMIT_MESSAGE="Automated deployment: $(date -u) ${GITHUB_SHA}"
if [[ ${INPUT_EMPTYCOMMITS} == "false" ]]; then
    git commit -m "${COMMIT_MESSAGE}" || skip
else
    git commit --allow-empty -m "${COMMIT_MESSAGE}"
fi

git push origin "${remote_branch}"
print_info "${GITHUB_SHA} was successfully deployed"
