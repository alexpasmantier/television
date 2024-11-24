_git-cliff() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="git__cliff"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        git__cliff)
            opts="-h -V -v -i -c -w -r -p -o -t -b -l -u -x -s --help --version --verbose --init --config --workdir --repository --include-path --exclude-path --tag-pattern --with-commit --with-tag-message --ignore-tags --count-tags --skip-commit --prepend --output --tag --bump --bumped-version --body --latest --current --unreleased --topo-order --use-branch-tags --no-exec --context --from-context --strip --sort --github-token --github-repo --gitlab-token --gitlab-repo --gitea-token --gitea-repo --bitbucket-token --bitbucket-repo [RANGE]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --init)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --workdir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -w)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --repository)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --include-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --exclude-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tag-pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --with-commit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --with-tag-message)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ignore-tags)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --count-tags)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --skip-commit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --prepend)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tag)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --bump)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --body)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --from-context)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --strip)
                    COMPREPLY=($(compgen -W "header footer all" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "header footer all" -- "${cur}"))
                    return 0
                    ;;
                --sort)
                    COMPREPLY=($(compgen -W "oldest newest" -- "${cur}"))
                    return 0
                    ;;
                --github-token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --github-repo)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab-token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab-repo)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitea-token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitea-repo)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --bitbucket-token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --bitbucket-repo)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _git-cliff -o nosort -o bashdefault -o default git-cliff
else
    complete -F _git-cliff -o bashdefault -o default git-cliff
fi
