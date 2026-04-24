internal static class OptionsTest
{
    private const string Script = """
title: Opts
---
NPC: Which way?
-> Left
    NPC: You went left.
-> Right
    NPC: You went right.
NPC: Done.
===
""";

    public static int Run()
    {
        var program = Helpers.Compile(Script);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine($"Options: runner_new failed: {Helpers.LastError()}");
            return 1;
        }

        if (Helpers.StartNode(runner, "Opts") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        // drain until the Options event
        var sawOptions = false;
        while (true)
        {
            var ev = Native.bubbles_runner_next_event(runner, out var ptr);
            if (ev == Native.Done) break;
            if (ev != Native.Ok)
            {
                Console.Error.WriteLine($"Options: next_event error: {Helpers.LastError()}");
                Native.bubbles_runner_free(runner);
                return 1;
            }
            var json = System.Runtime.InteropServices.Marshal.PtrToStringUTF8(ptr);
            Native.bubbles_string_free(ptr);
            if (json != null && json.Contains("\"kind\":\"Options\"", StringComparison.Ordinal))
            {
                sawOptions = true;
                break;
            }
        }

        if (!sawOptions)
        {
            Console.Error.WriteLine("Options: no Options event received");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        // select option 1 (Right)
        if (Native.bubbles_runner_select_option(runner, 1) != Native.Ok)
        {
            Console.Error.WriteLine($"Options: select_option failed: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        var remaining = Helpers.DrainAndCollect(runner);
        Native.bubbles_runner_free(runner);

        if (!remaining.Any(e => e.Contains("went right", StringComparison.OrdinalIgnoreCase)))
        {
            Console.Error.WriteLine("Options: expected 'went right' line after selecting option 1");
            return 1;
        }
        if (remaining.Any(e => e.Contains("went left", StringComparison.OrdinalIgnoreCase)))
        {
            Console.Error.WriteLine("Options: unexpected 'went left' line after selecting option 1");
            return 1;
        }

        return 0;
    }
}
