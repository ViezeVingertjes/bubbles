using System.Runtime.InteropServices;

internal static class SaveLoadTest
{
    private const string Script = """
title: Progress
---
<<set $step = 7>>
NPC: Checkpoint.
===
""";

    public static int Run()
    {
        // ── phase 1: run the script and capture snapshots ────────────────────

        var program1 = Helpers.Compile(Script);
        if (program1 == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program1, out var runner1) != Native.Ok || runner1 == nint.Zero)
        {
            Console.Error.WriteLine($"SaveLoad: runner_new failed: {Helpers.LastError()}");
            return 1;
        }

        if (Helpers.StartNode(runner1, "Progress") != 0) { Native.bubbles_runner_free(runner1); return 1; }

        if (!Helpers.DrainAll(runner1))
        {
            Console.Error.WriteLine($"SaveLoad: error draining events: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner1);
            return 1;
        }

        if (Native.bubbles_runner_snapshot_storage_json(runner1, out var storPtr) != Native.Ok)
        {
            Console.Error.WriteLine($"SaveLoad: snapshot_storage failed: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner1);
            return 1;
        }
        if (Native.bubbles_runner_snapshot_session_json(runner1, out var sesPtr) != Native.Ok)
        {
            Console.Error.WriteLine($"SaveLoad: snapshot_session failed: {Helpers.LastError()}");
            Native.bubbles_string_free(storPtr);
            Native.bubbles_runner_free(runner1);
            return 1;
        }

        var storageJson = Marshal.PtrToStringUTF8(storPtr)!;
        var sessionJson = Marshal.PtrToStringUTF8(sesPtr)!;
        Native.bubbles_string_free(storPtr);
        Native.bubbles_string_free(sesPtr);
        Native.bubbles_runner_free(runner1);

        // ── phase 2: restore onto a fresh runner and verify ──────────────────

        var program2 = Helpers.Compile(Script);
        if (program2 == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program2, out var runner2) != Native.Ok || runner2 == nint.Zero)
        {
            Console.Error.WriteLine($"SaveLoad: runner_new (restore) failed: {Helpers.LastError()}");
            return 1;
        }

        // storage must be restored before session
        using (var storPin = new Utf8Pin(storageJson))
        using (var sesPin  = new Utf8Pin(sessionJson))
        {
            if (Native.bubbles_runner_restore_storage_json(runner2, storPin.Ptr, storPin.Len) != Native.Ok)
            {
                Console.Error.WriteLine($"SaveLoad: restore_storage failed: {Helpers.LastError()}");
                Native.bubbles_runner_free(runner2);
                return 1;
            }
            if (Native.bubbles_runner_restore_session_json(runner2, sesPin.Ptr, sesPin.Len) != Native.Ok)
            {
                Console.Error.WriteLine($"SaveLoad: restore_session failed: {Helpers.LastError()}");
                Native.bubbles_runner_free(runner2);
                return 1;
            }
        }

        var val = Helpers.ReadVariable(runner2, "$step");
        Native.bubbles_runner_free(runner2);

        if (val != "{\"Number\":7.0}")
        {
            Console.Error.WriteLine($"SaveLoad: expected $step={{\"Number\":7.0}}, got {val}");
            return 1;
        }

        return 0;
    }
}
