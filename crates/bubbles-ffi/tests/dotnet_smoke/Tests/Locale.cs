internal static class LocaleTest
{
    private const string Script = """
title: L
---
Shopkeep: Hello #line:greet
===
""";

    public static int Run()
    {
        var program = Helpers.Compile(Script);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine($"Locale: runner_new failed: {Helpers.LastError()}");
            return 1;
        }

        using var locPin = new Utf8Pin("{\"greet\":\"Salut\"}");
        if (Native.bubbles_runner_set_locale_json(runner, locPin.Ptr, locPin.Len) != Native.Ok)
        {
            Console.Error.WriteLine($"Locale: set_locale_json failed: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (Helpers.StartNode(runner, "L") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        var events = Helpers.DrainAndCollect(runner);
        Native.bubbles_runner_free(runner);

        if (!events.Any(e => e.Contains("Salut", StringComparison.Ordinal)))
        {
            Console.Error.WriteLine("Locale: expected localised \"Salut\" in events");
            return 1;
        }

        return 0;
    }
}
