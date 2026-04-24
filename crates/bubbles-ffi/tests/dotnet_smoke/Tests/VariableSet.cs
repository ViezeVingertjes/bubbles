internal static class VariableSetTest
{
    // $code is declared so the compiler knows its type; the host overrides the default before start.
    private const string Script = """
title: Gate
---
<<declare $code = 0>>
<<if $code > 0>>
    NPC: Access granted.
<<else>>
    NPC: Access denied.
<<endif>>
===
""";

    public static int Run()
    {
        var program = Helpers.Compile(Script);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine($"VariableSet: runner_new failed: {Helpers.LastError()}");
            return 1;
        }

        // set $code = 1 before starting; verify the get round-trip first
        if (!Helpers.WriteVariable(runner, "$code", "1"))
        {
            Console.Error.WriteLine($"VariableSet: variable_set failed: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        var got = Helpers.ReadVariable(runner, "$code");
        if (got != "{\"Number\":1.0}")
        {
            Console.Error.WriteLine($"VariableSet: expected {{\"Number\":1.0}}, got {got}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (Helpers.StartNode(runner, "Gate") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        var events = Helpers.DrainAndCollect(runner);
        Native.bubbles_runner_free(runner);

        if (!events.Any(e => e.Contains("Access granted", StringComparison.Ordinal)))
        {
            Console.Error.WriteLine("VariableSet: expected 'Access granted' line");
            return 1;
        }
        if (events.Any(e => e.Contains("Access denied", StringComparison.Ordinal)))
        {
            Console.Error.WriteLine("VariableSet: unexpected 'Access denied' line (branch taken wrong)");
            return 1;
        }

        return 0;
    }
}
