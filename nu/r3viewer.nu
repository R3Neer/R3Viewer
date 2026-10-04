# R3Viewer for Nushell0.115.1. Load the packaged companion before this module.
# Binary locations belong to the package, never to the current directory.
const reader = if $nu.os-info.name == 'windows' { path self ../bin/r3viewer.exe } else { path self ../bin/r3viewer }

# View a path/URL argument or typed pipeline value. Read-only; no open override.
export def viewer [
    source?: string
    --inline(-i)
    --path(-p)
    --value(-v)
    --format: string = 'text'
] {
    # Keep encoder as the first command: any prelude statement loses PipelineData.
    r3viewer encode | ^$reader --nu-wire ...(if $inline { ['--inline'] } else { [] }) ...(if $path { ['--path'] } else { [] }) ...(if $value { ['--value'] } else { [] }) --format $format -- ...(if $source == null { [] } else { [$source] })
}
