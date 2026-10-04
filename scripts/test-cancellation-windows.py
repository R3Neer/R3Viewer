"""Windows reproduction harness; does not certify terminal rendering.

Run with Python and the exact r3viewer.exe to inspect. Each signal targets only
the child process group. Fixtures are synthetic and removed in finally.
The processing case uses the debug binary and records whether it remained alive
before the signal; it does not instrument/claim the exact processing instruction.
"""
import hashlib
import json
import pathlib
import signal
import subprocess
import sys
import tempfile
import time

exe = pathlib.Path(sys.argv[1])
print(json.dumps({"exe": str(exe), "sha256": hashlib.sha256(exe.read_bytes()).hexdigest().upper()}))
assert sys.platform == "win32", "Harness requires Windows console group signals"

with tempfile.TemporaryDirectory(prefix="r3viewer-cancel-adversary-") as folder:
    for kind in ("blocked_stdin", "blocked_stdout", "processing"):
        args = [str(exe), "-i"]
        if kind != "blocked_stdin":
            file = pathlib.Path(folder) / (kind + ".txt")
            file.write_bytes((b"a" if kind == "blocked_stdout" else b"\0") *
                             ((2 if kind == "blocked_stdout" else 32) * 1024 * 1024))
            args.append(str(file))
        child = subprocess.Popen(
            args,
            stdin=subprocess.PIPE if kind == "blocked_stdin" else subprocess.DEVNULL,
            stdout=subprocess.PIPE if kind != "processing" else subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP,
        )
        try:
            time.sleep(0.5)
            if child.poll() is not None:
                raise RuntimeError(f"{kind}: child exited before signal; inconclusive")
            child.send_signal(signal.CTRL_BREAK_EVENT)
            started = time.monotonic()
            child.wait(timeout=2)
            result = {"case": kind, "exit": child.returncode,
                      "latency_ms": round(1000 * (time.monotonic() - started))}
            print(json.dumps(result))
            assert child.returncode == 130, result
        finally:
            if child.poll() is None:
                child.kill()
            child.communicate()
