import tomllib
from pathlib import Path
from toml import dumps


CABLE_DIR = Path("./cable")
DOCS_DIR = Path("./docs/community")


def load_toml(path: Path) -> dict:
    """
    Load a channel file (`tomllib` supports mixed-type arrays, unlike `toml`).
    """
    with open(path, "rb") as f:
        return tomllib.load(f)


def format_requirement(requirement: str | list[str]) -> str:
    """
    Format a requirement, either a binary or a list of alternatives.
    """
    if isinstance(requirement, str):
        return f"`{requirement}`"
    first, *rest = requirement
    alternatives = ", ".join(f"`{alt}`" for alt in rest)
    return f"`{first}` (or {alternatives})" if rest else f"`{first}`"


def generate_cable_docs(os_name: str) -> str:
    """
    Generate documentation for community channels.
    """
    cable_dir = CABLE_DIR.joinpath(os_name)
    cable_dir.mkdir(parents=True, exist_ok=True)

    docs = f"""
# Community Channels ({os_name})
"""

    channels = map(load_toml, sorted(cable_dir.glob("*.toml")))

    for channel in channels:
        channel_name = channel["metadata"]["name"]
        channel_desc = channel["metadata"]["description"]
        channel_requirements = channel["metadata"].get("requirements", [])

        docs += f"""
### *{channel_name}*

{channel_desc}

"""
        # prefer OS-specific image, fallback to generic
        img_paths = [
            Path(f"./assets/channels/{os_name}/{channel_name}.png"),
            Path(f"./assets/channels/{channel_name}.png"),
        ]
        for img_path in img_paths:
            if img_path.exists():
                docs += f"![tv running the {channel_name} channel](../../{img_path})\n"
                break

        docs += f"""**Requirements:** {", ".join(map(format_requirement, channel_requirements)) if channel_requirements else "*None*"}

**Code:** *{channel_name}.toml*

```toml
{dumps(channel)}
```

"""
        docs += "\n---\n"

    return docs


if __name__ == "__main__":
    for os_name in ("unix", "windows"):
        docs_content = generate_cable_docs(os_name)
        docs_file = DOCS_DIR.joinpath(f"channels-{os_name}.md")
        # write the new docs
        with open(docs_file, "w+", encoding="utf-8") as f:
            f.write(docs_content)

        print(
            f"Generated documentation for {os_name} community channels at {docs_file}"
        )
