internal static class BasicTest
{
    private const string Script = """
title: Start
---
Alice: Hi
===
""";

    public static int Run()
    {
        var program = Helpers.Compile(Script);
        if (program == nint.Zero) return 1;

        using var nodePin = new Utf8Pin("Start");
        if (Native.bubbles_program_node_exists(program, nodePin.Ptr, nodePin.Len, out var exists) != Native.Ok
            || exists != 1)
        {
            Console.Error.WriteLine("Basic: bubbles_program_node_exists failed");
            Native.bubbles_program_free(program);
            return 1;
        }

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine($"Basic: runner_new failed: {Helpers.LastError()}");
            return 1;
        }

        if (Helpers.StartNode(runner, "Start") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (!Helpers.DrainUntilLine(runner))
        {
            Console.Error.WriteLine("Basic: no Line event received");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        Native.bubbles_runner_free(runner);
        return 0;
    }
}
