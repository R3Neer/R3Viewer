# Controlled Crossterm0.29.0 patch

Upstream registry archive and each original file are pinned in R3VIEWER_UPSTREAM.json. Original MIT license retained. This copy belongs only to R3Viewer; no installed terminal, Nu or Cargo registry is modified.

Windows KEY_EVENT VK_MENU release with nonzero Unicode commits an Alt-numpad character. Upstream identifies this exact case but labels it Release, so consumers that ignore ordinary releases lose the character. The patch labels only that committed BMP character Press. Ordinary releases, modifiers, Unix and UTF-16 surrogate handling remain upstream. Tests distinguish commit from ordinary press/release and retain AltGr modifier identity. Non-BMP keyboard input is validated separately; this patch does not claim to repair unobserved surrogate issues.

Upgrade gate: compare against the recorded archive, re-evaluate the upstream branch, run these parser tests and independent ConPTY Unicode/AltGr/normal-key/cancellation acceptance before removing or rebasing this patch.

Cargo.toml additionally declares an empty workspace so the excluded controlled copy can run its own parser tests. This does not alter library features or dependencies.

R4 additionally normalizes an actual printable BMP UnicodeChar with the exact Windows AltGr sides (right Alt and left Ctrl, without left Alt or right Ctrl) to text modifiers, preserving Shift and event kind. Each required state bit is checked separately because ControlKeyState::has_state accepts any bit. VK-derived/control characters and ordinary Ctrl/Alt combinations remain unchanged. This permits logical `/` input on layouts where Windows delivers it through AltGr without enabling all Ctrl+Alt application shortcuts. Supplementary UTF-16 handling stays upstream and is tested separately.
