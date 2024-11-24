complete -c git-cliff -s i -l init -d 'Writes the default configuration file to cliff.toml' -r
complete -c git-cliff -s c -l config -d 'Sets the configuration file' -r -F
complete -c git-cliff -s w -l workdir -d 'Sets the working directory' -r -F
complete -c git-cliff -s r -l repository -d 'Sets the git repository' -r -F
complete -c git-cliff -l include-path -d 'Sets the path to include related commits' -r
complete -c git-cliff -l exclude-path -d 'Sets the path to exclude related commits' -r
complete -c git-cliff -l tag-pattern -d 'Sets the regex for matching git tags' -r
complete -c git-cliff -l with-commit -d 'Sets custom commit messages to include in the changelog' -r
complete -c git-cliff -l with-tag-message -d 'Sets custom message for the latest release' -r
complete -c git-cliff -l ignore-tags -d 'Sets the tags to ignore in the changelog' -r
complete -c git-cliff -l count-tags -d 'Sets the tags to count in the changelog' -r
complete -c git-cliff -l skip-commit -d 'Sets commits that will be skipped in the changelog' -r
complete -c git-cliff -s p -l prepend -d 'Prepends entries to the given changelog file' -r -F
complete -c git-cliff -s o -l output -d 'Writes output to the given file' -r -F
complete -c git-cliff -s t -l tag -d 'Sets the tag for the latest version' -r
complete -c git-cliff -l bump -d 'Bumps the version for unreleased changes. Optionally with specified version' -r
complete -c git-cliff -s b -l body -d 'Sets the template for the changelog body' -r
complete -c git-cliff -l from-context -d 'Generates changelog from a JSON context' -r -F
complete -c git-cliff -s s -l strip -d 'Strips the given parts from the changelog' -r -f -a "{header\t'',footer\t'',all\t''}"
complete -c git-cliff -l sort -d 'Sets sorting of the commits inside sections' -r -f -a "{oldest\t'',newest\t''}"
complete -c git-cliff -l github-token -d 'Sets the GitHub API token' -r
complete -c git-cliff -l github-repo -d 'Sets the GitHub repository' -r
complete -c git-cliff -l gitlab-token -d 'Sets the GitLab API token' -r
complete -c git-cliff -l gitlab-repo -d 'Sets the GitLab repository' -r
complete -c git-cliff -l gitea-token -d 'Sets the Gitea API token' -r
complete -c git-cliff -l gitea-repo -d 'Sets the Gitea repository' -r
complete -c git-cliff -l bitbucket-token -d 'Sets the Bitbucket API token' -r
complete -c git-cliff -l bitbucket-repo -d 'Sets the Bitbucket repository' -r
complete -c git-cliff -s h -l help -d 'Prints help information'
complete -c git-cliff -s V -l version -d 'Prints version information'
complete -c git-cliff -s v -l verbose -d 'Increases the logging verbosity'
complete -c git-cliff -l bumped-version -d 'Prints bumped version for unreleased changes'
complete -c git-cliff -s l -l latest -d 'Processes the commits starting from the latest tag'
complete -c git-cliff -l current -d 'Processes the commits that belong to the current tag'
complete -c git-cliff -s u -l unreleased -d 'Processes the commits that do not belong to a tag'
complete -c git-cliff -l topo-order -d 'Sorts the tags topologically'
complete -c git-cliff -l use-branch-tags -d 'Include only the tags that belong to the current branch'
complete -c git-cliff -l no-exec -d 'Disables the external command execution'
complete -c git-cliff -s x -l context -d 'Prints changelog context as JSON'
