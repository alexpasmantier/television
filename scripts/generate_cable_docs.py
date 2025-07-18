from pathlib import Path
from toml import load as load_toml, dumps


CABLE_DIR = Path("./cable")
DOCS_DIR = Path("./docs/01-Users")


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
        img_path = Path(f"./assets/channels/{channel_name}.png")
        if img_path.exists():
            docs += f"![tv running the {channel_name} channel](../../{img_path})\n"

        docs += f"""**Requirements:** {", ".join((f"`{req}`" for req in channel_requirements)) if channel_requirements else "*None*"}

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
        docs_file = DOCS_DIR.joinpath(f"10-community-channels-{os_name}.md")
        # write the new docs
        with open(docs_file, "w+", encoding="utf-8") as f:
            f.write(docs_content)

        print(
            f"Generated documentation for {os_name} community channels at {docs_file}"
        )
