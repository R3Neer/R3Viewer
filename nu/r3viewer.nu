# R3Viewer for Nushell0.115.1. Load the packaged companion before this module.
# Binary locations belong to the package, never to the current directory.
const reader = if $nu.os-info.name == 'windows' { path self ../bin/r3viewer.exe } else { path self ../bin/r3viewer }

const __r3v_names = {
  "green": {
    "ansi": 2
  },
  "g": {
    "ansi": 2
  },
  "green_bold": {
    "ansi": 2
  },
  "gb": {
    "ansi": 2
  },
  "green_underline": {
    "ansi": 2
  },
  "gu": {
    "ansi": 2
  },
  "green_italic": {
    "ansi": 2
  },
  "gi": {
    "ansi": 2
  },
  "green_dimmed": {
    "ansi": 2
  },
  "gd": {
    "ansi": 2
  },
  "green_reverse": {
    "ansi": 2
  },
  "gr": {
    "ansi": 2
  },
  "light_green": {
    "ansi": 10
  },
  "lg": {
    "ansi": 10
  },
  "light_green_bold": {
    "ansi": 10
  },
  "lgb": {
    "ansi": 10
  },
  "light_green_underline": {
    "ansi": 10
  },
  "lgu": {
    "ansi": 10
  },
  "light_green_italic": {
    "ansi": 10
  },
  "lgi": {
    "ansi": 10
  },
  "light_green_dimmed": {
    "ansi": 10
  },
  "lgd": {
    "ansi": 10
  },
  "light_green_reverse": {
    "ansi": 10
  },
  "lgr": {
    "ansi": 10
  },
  "red": {
    "ansi": 1
  },
  "r": {
    "ansi": 1
  },
  "red_bold": {
    "ansi": 1
  },
  "rb": {
    "ansi": 1
  },
  "red_underline": {
    "ansi": 1
  },
  "ru": {
    "ansi": 1
  },
  "red_italic": {
    "ansi": 1
  },
  "ri": {
    "ansi": 1
  },
  "red_dimmed": {
    "ansi": 1
  },
  "rd": {
    "ansi": 1
  },
  "red_reverse": {
    "ansi": 1
  },
  "rr": {
    "ansi": 1
  },
  "light_red": {
    "ansi": 9
  },
  "lr": {
    "ansi": 9
  },
  "light_red_bold": {
    "ansi": 9
  },
  "lrb": {
    "ansi": 9
  },
  "light_red_underline": {
    "ansi": 9
  },
  "lru": {
    "ansi": 9
  },
  "light_red_italic": {
    "ansi": 9
  },
  "lri": {
    "ansi": 9
  },
  "light_red_dimmed": {
    "ansi": 9
  },
  "lrd": {
    "ansi": 9
  },
  "light_red_reverse": {
    "ansi": 9
  },
  "lrr": {
    "ansi": 9
  },
  "blue": {
    "ansi": 4
  },
  "b": {
    "ansi": 4
  },
  "blue_bold": {
    "ansi": 4
  },
  "bb": {
    "ansi": 4
  },
  "blue_underline": {
    "ansi": 4
  },
  "bu": {
    "ansi": 4
  },
  "blue_italic": {
    "ansi": 4
  },
  "bi": {
    "ansi": 4
  },
  "blue_dimmed": {
    "ansi": 4
  },
  "bd": {
    "ansi": 4
  },
  "blue_reverse": {
    "ansi": 4
  },
  "br": {
    "ansi": 4
  },
  "light_blue": {
    "ansi": 12
  },
  "lu": {
    "ansi": 12
  },
  "light_blue_bold": {
    "ansi": 12
  },
  "lub": {
    "ansi": 12
  },
  "light_blue_underline": {
    "ansi": 12
  },
  "luu": {
    "ansi": 12
  },
  "light_blue_italic": {
    "ansi": 12
  },
  "lui": {
    "ansi": 12
  },
  "light_blue_dimmed": {
    "ansi": 12
  },
  "lud": {
    "ansi": 12
  },
  "light_blue_reverse": {
    "ansi": 12
  },
  "lur": {
    "ansi": 12
  },
  "black": {
    "ansi": 0
  },
  "k": {
    "ansi": 0
  },
  "black_bold": {
    "ansi": 0
  },
  "kb": {
    "ansi": 0
  },
  "black_underline": {
    "ansi": 0
  },
  "ku": {
    "ansi": 0
  },
  "black_italic": {
    "ansi": 0
  },
  "ki": {
    "ansi": 0
  },
  "black_dimmed": {
    "ansi": 0
  },
  "kd": {
    "ansi": 0
  },
  "black_reverse": {
    "ansi": 0
  },
  "kr": {
    "ansi": 0
  },
  "light_gray": {
    "ansi": 15
  },
  "ligr": {
    "ansi": 15
  },
  "light_gray_bold": {
    "ansi": 15
  },
  "ligrb": {
    "ansi": 15
  },
  "light_gray_underline": {
    "ansi": 15
  },
  "ligru": {
    "ansi": 15
  },
  "light_gray_italic": {
    "ansi": 15
  },
  "ligri": {
    "ansi": 15
  },
  "light_gray_dimmed": {
    "ansi": 15
  },
  "ligrd": {
    "ansi": 15
  },
  "light_gray_reverse": {
    "ansi": 15
  },
  "ligrr": {
    "ansi": 15
  },
  "yellow": {
    "ansi": 3
  },
  "y": {
    "ansi": 3
  },
  "yellow_bold": {
    "ansi": 3
  },
  "yb": {
    "ansi": 3
  },
  "yellow_underline": {
    "ansi": 3
  },
  "yu": {
    "ansi": 3
  },
  "yellow_italic": {
    "ansi": 3
  },
  "yi": {
    "ansi": 3
  },
  "yellow_dimmed": {
    "ansi": 3
  },
  "yd": {
    "ansi": 3
  },
  "yellow_reverse": {
    "ansi": 3
  },
  "yr": {
    "ansi": 3
  },
  "light_yellow": {
    "ansi": 11
  },
  "ly": {
    "ansi": 11
  },
  "light_yellow_bold": {
    "ansi": 11
  },
  "lyb": {
    "ansi": 11
  },
  "light_yellow_underline": {
    "ansi": 11
  },
  "lyu": {
    "ansi": 11
  },
  "light_yellow_italic": {
    "ansi": 11
  },
  "lyi": {
    "ansi": 11
  },
  "light_yellow_dimmed": {
    "ansi": 11
  },
  "lyd": {
    "ansi": 11
  },
  "light_yellow_reverse": {
    "ansi": 11
  },
  "lyr": {
    "ansi": 11
  },
  "purple": {
    "ansi": 5
  },
  "p": {
    "ansi": 5
  },
  "purple_bold": {
    "ansi": 5
  },
  "pb": {
    "ansi": 5
  },
  "purple_underline": {
    "ansi": 5
  },
  "pu": {
    "ansi": 5
  },
  "purple_italic": {
    "ansi": 5
  },
  "pi": {
    "ansi": 5
  },
  "purple_dimmed": {
    "ansi": 5
  },
  "pd": {
    "ansi": 5
  },
  "purple_reverse": {
    "ansi": 5
  },
  "pr": {
    "ansi": 5
  },
  "light_purple": {
    "ansi": 13
  },
  "lp": {
    "ansi": 13
  },
  "light_purple_bold": {
    "ansi": 13
  },
  "lpb": {
    "ansi": 13
  },
  "light_purple_underline": {
    "ansi": 13
  },
  "lpu": {
    "ansi": 13
  },
  "light_purple_italic": {
    "ansi": 13
  },
  "lpi": {
    "ansi": 13
  },
  "light_purple_dimmed": {
    "ansi": 13
  },
  "lpd": {
    "ansi": 13
  },
  "light_purple_reverse": {
    "ansi": 13
  },
  "lpr": {
    "ansi": 13
  },
  "magenta": {
    "ansi": 5
  },
  "m": {
    "ansi": 5
  },
  "magenta_bold": {
    "ansi": 5
  },
  "mb": {
    "ansi": 5
  },
  "magenta_underline": {
    "ansi": 5
  },
  "mu": {
    "ansi": 5
  },
  "magenta_italic": {
    "ansi": 5
  },
  "mi": {
    "ansi": 5
  },
  "magenta_dimmed": {
    "ansi": 5
  },
  "md": {
    "ansi": 5
  },
  "magenta_reverse": {
    "ansi": 5
  },
  "mr": {
    "ansi": 5
  },
  "light_magenta": {
    "ansi": 13
  },
  "lm": {
    "ansi": 13
  },
  "light_magenta_bold": {
    "ansi": 13
  },
  "lmb": {
    "ansi": 13
  },
  "light_magenta_underline": {
    "ansi": 13
  },
  "lmu": {
    "ansi": 13
  },
  "light_magenta_italic": {
    "ansi": 13
  },
  "lmi": {
    "ansi": 13
  },
  "light_magenta_dimmed": {
    "ansi": 13
  },
  "lmd": {
    "ansi": 13
  },
  "light_magenta_reverse": {
    "ansi": 13
  },
  "lmr": {
    "ansi": 13
  },
  "cyan": {
    "ansi": 6
  },
  "c": {
    "ansi": 6
  },
  "cyan_bold": {
    "ansi": 6
  },
  "cb": {
    "ansi": 6
  },
  "cyan_underline": {
    "ansi": 6
  },
  "cu": {
    "ansi": 6
  },
  "cyan_italic": {
    "ansi": 6
  },
  "ci": {
    "ansi": 6
  },
  "cyan_dimmed": {
    "ansi": 6
  },
  "cd": {
    "ansi": 6
  },
  "cyan_reverse": {
    "ansi": 6
  },
  "cr": {
    "ansi": 6
  },
  "light_cyan": {
    "ansi": 14
  },
  "lc": {
    "ansi": 14
  },
  "light_cyan_bold": {
    "ansi": 14
  },
  "lcb": {
    "ansi": 14
  },
  "light_cyan_underline": {
    "ansi": 14
  },
  "lcu": {
    "ansi": 14
  },
  "light_cyan_italic": {
    "ansi": 14
  },
  "lci": {
    "ansi": 14
  },
  "light_cyan_dimmed": {
    "ansi": 14
  },
  "lcd": {
    "ansi": 14
  },
  "light_cyan_reverse": {
    "ansi": 14
  },
  "lcr": {
    "ansi": 14
  },
  "white": {
    "ansi": 7
  },
  "w": {
    "ansi": 7
  },
  "white_bold": {
    "ansi": 7
  },
  "wb": {
    "ansi": 7
  },
  "white_underline": {
    "ansi": 7
  },
  "wu": {
    "ansi": 7
  },
  "white_italic": {
    "ansi": 7
  },
  "wi": {
    "ansi": 7
  },
  "white_dimmed": {
    "ansi": 7
  },
  "wd": {
    "ansi": 7
  },
  "white_reverse": {
    "ansi": 7
  },
  "wr": {
    "ansi": 7
  },
  "dark_gray": {
    "ansi": 8
  },
  "dgr": {
    "ansi": 8
  },
  "dark_gray_bold": {
    "ansi": 8
  },
  "dgrb": {
    "ansi": 8
  },
  "dark_gray_underline": {
    "ansi": 8
  },
  "dgru": {
    "ansi": 8
  },
  "dark_gray_italic": {
    "ansi": 8
  },
  "dgri": {
    "ansi": 8
  },
  "dark_gray_dimmed": {
    "ansi": 8
  },
  "dgrd": {
    "ansi": 8
  },
  "dark_gray_reverse": {
    "ansi": 8
  },
  "dgrr": {
    "ansi": 8
  },
  "default": {
    "default": true
  },
  "def": {
    "default": true
  },
  "default_bold": {
    "default": true
  },
  "defb": {
    "default": true
  },
  "default_underline": {
    "default": true
  },
  "defu": {
    "default": true
  },
  "default_italic": {
    "default": true
  },
  "defi": {
    "default": true
  },
  "default_dimmed": {
    "default": true
  },
  "defd": {
    "default": true
  },
  "default_reverse": {
    "default": true
  },
  "defr": {
    "default": true
  }
}

# Pure presentation argument; these commands never receive the viewer's PipelineData.
def __r3v_fg [candidate: any] {
    match $candidate {
        {} => null
        [..] => null
        _ => {
            if ($candidate | describe) != 'string' { return null }
            if ($candidate | str length --utf-8-bytes) > 32 { return null }
            let named = ($__r3v_names | get -o $candidate)
            if $named != null { return $named }
            if $candidate =~ '^#[0-9a-fA-F]{6}$' {
                return {rgb: [
                    ($candidate | str substring 1..<3 | into int --radix 16)
                    ($candidate | str substring 3..<5 | into int --radix 16)
                    ($candidate | str substring 5..<7 | into int --radix 16)
                ]}
            }
            null
        }
    }
}
def __r3v_style [candidate: any] {
    match $candidate {
        {fg: $fg} => (__r3v_fg $fg)
        {} => null
        _ => (__r3v_fg $candidate)
    }
}
def __r3v_palette [] {
    let colours = $env.config.color_config
    mut roles = {}
    for slot in [
        {role: 'heading' key: 'header'} {role: 'accent' key: 'cell-path'}
        {role: 'secondary' key: 'separator'} {role: 'value' key: 'string'}
        {role: 'success' key: 'bool'} {role: 'error' key: 'error'}
        {role: 'process' key: 'int'}
    ] {
        let fg = (__r3v_style ($colours | get -o $slot.key))
        if $fg != null { $roles = ($roles | upsert $slot.role $fg) }
    }
    let metadata = ({v: 1 roles: $roles} | to json --raw)
    if ($metadata | str length --utf-8-bytes) > 1024 {
        error make {msg: 'Nu palette metadata exceeds internal limit'}
    }
    $metadata
}

# View a path/URL argument or typed pipeline value. Read-only; no open override.
export def viewer [
    source?: string
    --inline(-i)
    --path(-p)
    --value(-v)
    # auto/text/code/markdown/toml/json/yaml/xml; markdown/xml require textual data or a real path.
    --format: string = 'auto'
    --language: string
    --structure
    --config: string
    --colour: string
    --theme: string
    --line-numbers(-n)
    --no-line-numbers
] {
    # Keep encoder as the first command: any prelude statement loses PipelineData.
    r3viewer encode | ^$reader --nu-wire --nu-palette (__r3v_palette) ...(if $inline { ['--inline'] } else { [] }) ...(if $path { ['--path'] } else { [] }) ...(if $value { ['--value'] } else { [] }) --format $format ...(if $structure { ['--structure'] } else { [] }) ...(if $language == null { [] } else { ['--language' $language] }) ...(if $config == null { [] } else { ['--config' $config] }) ...(if $colour == null { [] } else { ['--colour' $colour] }) ...(if $theme == null { [] } else { ['--theme' $theme] }) ...(if $line_numbers { ['--line-numbers'] } else { [] }) ...(if $no_line_numbers { ['--no-line-numbers'] } else { [] }) -- ...(if $source == null { [] } else { [$source] })
}
