#!/usr/bin/env python3

# This script runs through this git project's git commit history and makes
# sure that all Cargo.toml package versions match the current commit's tag
# version, e.g. tag "tutorial1.3" should have Cargo.toml versions set to
# "0.1.3"

import os
import toml # pip3 install toml
from subprocess import call, CalledProcessError
from commit_history import *

git_status = check_output(["git", "status", "-s"]).decode('utf-8')
if git_status:
    raise Exception(
        'Edits are preventing version checking from proceeding:\n{0}'.format(
            git_status))


def verify_package(package: str, version: Version) -> bool:
    try:
        pkgid = check_output(["cargo", "pkgid", "--package",
                              package]).decode('utf-8').strip()
        print('Checking "{0}" set to version "{1}"...'.format(
            package, version))
        if not pkgid.endswith(str(version)):
            print('[ERROR] Package {0} is not set to version 0.{1}'.format(
                package, version))
            print('        pkgid = "{0}"'.format(pkgid))
            return False
    except CalledProcessError:
        # Package does not exist yet, that's fine. It will be added in a later CL.
        pass

    return True


# Verify all packages across all commits
def verify_all() -> bool:
    all_verified = True
    # Checkout from oldest to newest
    for commit in CommitHistory.from_git_logs()[::-1]:
        print('Verifying commit [{0}]: "{1}"'.format(commit.git_hash,
                                                     commit.message))
        call(['git', 'checkout', '-q', commit.git_hash])

        if commit.version is not None:
            dirs = [d for d in os.listdir('.') if os.path.isdir(d)]
            for dir in dirs:
                config = os.path.join(dir, 'Cargo.toml')
                if os.path.isfile(config):
                    config = toml.load(config)
                    package = config['package']['name']
                    version = config['package']['version']

                    if version != "0.{0}".format(commit.version):
                        print(
                            '[ERROR] Package {0} is set to {1}, not version 0.{2}'
                            .format(package, version, commit.version))
                        all_verified = False
        else:
            print("Skipping commit as it has no version information")

    return all_verified


result = verify_all()
call(['git', 'checkout', '-q', 'master'])

if not result:
    raise Exception("Failed to verify all commits. Please see logs above.")
