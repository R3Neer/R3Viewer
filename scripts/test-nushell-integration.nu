# Copied into a private package/tests by test-nushell-integration.ps1.
use ../nu/r3viewer.nu viewer
use std/assert

def main [config: path] {
    let text = do { 'literal α🙂' | viewer -i --colour never --config $config } | complete
    assert equal $text.exit_code 0
    assert equal ($text.stdout | str trim --right) 'literal α🙂'
    assert equal $text.stderr ''
    let record = do { {name: 'Ana', n: 42, empty: null, flag: true} | viewer -i --colour never --config $config } | complete
    assert equal $record.exit_code 0
    assert equal ($record.stdout | str trim --right) 'record{"name": string("Ana"), "n": int(42), "empty": nothing, "flag": bool(true)}'
    let binary = do { 0x[00 ff 01] | viewer -i --colour never --config $config } | complete
    assert equal $binary.exit_code 0
    assert equal ($binary.stdout | str trim --right) 'binary(0x00ff01)'
    let stream = do { [1 2 3] | each {|x| $x } | viewer -i --colour never --config $config } | complete
    assert equal $stream.exit_code 0
    assert equal ($stream.stdout | str trim --right) 'list_stream[int(1), int(2), int(3)]'
    let nothing = do { null | viewer -i --colour never --config $config } | complete
    assert equal $nothing.exit_code 0
    assert equal ($nothing.stdout | str trim --right) 'nothing'
    let empty = do { '' | viewer -i --colour never --config $config } | complete
    assert equal $empty.exit_code 0
    assert equal $empty.stdout ''
    let bytesource = ($config | path dirname | path join 'tests' 'source.txt')
    let byteinput = do { open --raw $bytesource | into string | viewer -i --colour never --config $config } | complete
    assert equal $byteinput.exit_code 0
    assert equal ($byteinput.stdout | str trim --right) 'stream α'
    let csvsource = ($config | path dirname | path join 'tests' 'source.csv')
    let transformed = do { open $csvsource | get n | each {|n| ($n | into int) + 1} | viewer -i --colour never --config $config } | complete
    assert equal $transformed.exit_code 0
    assert equal ($transformed.stdout | str trim --right) 'list_stream[int(43)]'
    # Abandon actual plugin byte outputs, then use the same plugin session again.
    for _ in 1..5 { 1..100000 | each {|x| $x} | r3viewer encode | take 8 | ignore }
    let reused = do { 99 | viewer -i --colour never --config $config } | complete
    assert equal $reused.exit_code 0
    assert equal ($reused.stdout | str trim --right) 'int(99)'
    $env.config.color_config.string = '#123456'
    let palette = do { 'palette' | viewer -i --colour never --config $config } | complete
    assert equal $palette.exit_code 0
    assert equal ($palette.stdout | str trim --right) 'palette'
    assert (not ($palette.stdout | str contains (char --integer 27)))
    # Direct arguments keep file/value discrimination across the SDK migration.
    let value = do { viewer -i --value 'argument α' --colour never --config $config } | complete
    assert equal $value.exit_code 0
    assert equal ($value.stdout | str trim --right) 'argument α'
    print 'PASS: typed values, actual byte/list streams and transformed metadata, empty/nothing, abandonment/reuse, palette redirect, literal argument'
}
