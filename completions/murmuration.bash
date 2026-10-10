_murmuration() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="murmuration"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        murmuration)
            opts="-h -V --analyze --turns --agitation --probe --zigzag-k --turning --falcon --colors --screensaver --no-mouse --no-arc --arc-minutes --birds --seed --turn-side --turn-angle --turn-window --turn-min --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --analyze)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --zigzag-k)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --turning)
                    COMPREPLY=($(compgen -W "base inertial" -- "${cur}"))
                    return 0
                    ;;
                --colors)
                    COMPREPLY=($(compgen -W "auto truecolor ansi256 none" -- "${cur}"))
                    return 0
                    ;;
                --arc-minutes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --birds)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --seed)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --turn-side)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --turn-angle)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --turn-window)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --turn-min)
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
    complete -F _murmuration -o nosort -o bashdefault -o default murmuration
else
    complete -F _murmuration -o bashdefault -o default murmuration
fi
