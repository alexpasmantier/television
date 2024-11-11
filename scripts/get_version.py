import re


WORKSPACE_CARGO_PATH = "Cargo.toml"
VERSION_RE = re.compile(r'version\s+=\s+"(\d+\.\d+\.\d+)"')


def get_version() -> str:
    with open(WORKSPACE_CARGO_PATH, "r") as f:
        lines = f.readlines()
        l = 0
        for i, line in enumerate(lines):
            if line == "[dependencies]":
                l = i
                break
        for i in range(l, len(lines)):
            if lines[i].startswith("television-"):
                return VERSION_RE.search(lines[i]).group(1)
    return "0.0.0"


def bump_version(version: str) -> str:
    major, minor, patch = version.split(".")
    return f"{major}.{minor}.{int(patch) + 1}"

if __name__ == "__main__":
    print(bump_version(get_version()))

