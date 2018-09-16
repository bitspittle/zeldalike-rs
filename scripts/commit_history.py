# Utility classes for parsing relevant bits out of the git history for this project

import re
from subprocess import check_output
from typing import List


class Version:
    def __init__(self, major, minor):
        self.major = major
        self.minor = minor

    def __str__(self):
        return "{0}.{1}".format(self.major, self.minor)


class CommitEntry:
    def __init__(self, git_hash: str, message: str, version: Version = None):
        self.git_hash = git_hash
        self.message = message
        self.version = version

    def __str__(self):
        return "{0}: {1}".format(self.git_hash, self.message)


class CommitHistory:
    @staticmethod
    def from_git_logs() -> List[CommitEntry]:
        commits = []

        # Show log with "hash: commit header" format
        # Example output: "edefcac1c7882290d12233d11725e7053b8dab24: Tutorial 1.1"
        git_line_pattern = re.compile(r"([^\s]+): (.*)")
        git_version_pattern = re.compile("Tutorial (\\d+)(\\.(\\d+))?")

        git_output = check_output(["git", "log", '''--pretty=format:%H: %s'''])
        # Last line is "Initial commit" so slice it out
        for git_line in map(lambda bytes: bytes.decode('utf-8'),
                            git_output.splitlines()[0:-1]):
            print('Parsing: "{0}"...'.format(git_line))
            line_match = git_line_pattern.match(git_line)
            hash = line_match.group(1)
            message = line_match.group(2)

            version = None
            print('  Parsing: "{0}"...'.format(message))
            version_match = git_version_pattern.match(message)
            if version_match is not None:
                major = int(version_match.group(1))
                minor = int(version_match.group(3) or "0")
                version = Version(major, minor)

            commits.append(CommitEntry(hash, message, version))

        return commits
