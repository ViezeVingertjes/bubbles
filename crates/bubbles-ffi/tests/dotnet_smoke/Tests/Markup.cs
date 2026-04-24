using System.Text.Json;
using System.Text.Json.Nodes;

internal static class MarkupTest
{
    private const string Script = """
title: Start
---
[wave]Hello[/wave] there!
-> [b]Fight[/b]
-> Run
===
""";

    public static int Run()
    {
        var program = Helpers.Compile(Script);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new(program, out var runner) != Native.Ok || runner == nint.Zero)
        {
            Console.Error.WriteLine($"Markup: runner_new failed: {Helpers.LastError()}");
            Native.bubbles_program_free(program);
            return 1;
        }

        if (Helpers.StartNode(runner, "Start") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        // ── Line event: spans on "[wave]Hello[/wave] there!" ─────────────────
        string? lineJson = null;
        while (lineJson == null)
        {
            var rc = Native.bubbles_runner_next_event(runner, out var ptr);
            if (rc == Native.Done || rc != Native.Ok) break;
            var json = System.Runtime.InteropServices.Marshal.PtrToStringUTF8(ptr);
            Native.bubbles_string_free(ptr);
            if (json != null && json.Contains("\"kind\":\"Line\"", StringComparison.Ordinal))
                lineJson = json;
        }

        if (lineJson == null)
        {
            Console.Error.WriteLine("Markup: no Line event received");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (!VerifyLineSpans(lineJson))
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        // ── Options event: spans on "[b]Fight[/b]" ───────────────────────────
        string? optsJson = null;
        while (optsJson == null)
        {
            var rc = Native.bubbles_runner_next_event(runner, out var ptr);
            if (rc == Native.Done || rc != Native.Ok) break;
            var json = System.Runtime.InteropServices.Marshal.PtrToStringUTF8(ptr);
            Native.bubbles_string_free(ptr);
            if (json != null && json.Contains("\"kind\":\"Options\"", StringComparison.Ordinal))
                optsJson = json;
        }

        if (optsJson == null)
        {
            Console.Error.WriteLine("Markup: no Options event received");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (!VerifyOptionSpans(optsJson))
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        // Select an option and drain to completion.
        Native.bubbles_runner_select_option(runner, 0);
        Helpers.DrainAll(runner);
        Native.bubbles_runner_free(runner);
        return 0;
    }

    private static bool VerifyLineSpans(string json)
    {
        JsonNode? root;
        try { root = JsonNode.Parse(json); }
        catch { Console.Error.WriteLine($"Markup: failed to parse Line JSON: {json}"); return false; }

        var text = root?["text"]?.GetValue<string>();
        if (text != "Hello there!")
        {
            Console.Error.WriteLine($"Markup: expected text 'Hello there!', got '{text}'");
            return false;
        }

        var spans = root?["spans"]?.AsArray();
        if (spans == null || spans.Count != 1)
        {
            Console.Error.WriteLine($"Markup: expected 1 span on Line, got {spans?.Count ?? -1}; json={json}");
            return false;
        }

        var span = spans[0];
        var name   = span?["name"]?.GetValue<string>();
        var start  = span?["start"]?.GetValue<int>();
        var length = span?["length"]?.GetValue<int>();

        if (name != "wave" || start != 0 || length != 5)
        {
            Console.Error.WriteLine(
                $"Markup: unexpected Line span: name={name} start={start} length={length}");
            return false;
        }

        return true;
    }

    private static bool VerifyOptionSpans(string json)
    {
        JsonNode? root;
        try { root = JsonNode.Parse(json); }
        catch { Console.Error.WriteLine($"Markup: failed to parse Options JSON: {json}"); return false; }

        var options = root?["options"]?.AsArray();
        if (options == null || options.Count < 1)
        {
            Console.Error.WriteLine("Markup: no options in Options event");
            return false;
        }

        var firstOpt = options[0];
        var optText = firstOpt?["text"]?.GetValue<string>();
        if (optText != "Fight")
        {
            Console.Error.WriteLine($"Markup: expected option text 'Fight', got '{optText}'");
            return false;
        }

        var spans = firstOpt?["spans"]?.AsArray();
        if (spans == null || spans.Count != 1)
        {
            Console.Error.WriteLine($"Markup: expected 1 span on option, got {spans?.Count ?? -1}");
            return false;
        }

        var name   = spans[0]?["name"]?.GetValue<string>();
        var start  = spans[0]?["start"]?.GetValue<int>();
        var length = spans[0]?["length"]?.GetValue<int>();

        if (name != "b" || start != 0 || length != 5)
        {
            Console.Error.WriteLine(
                $"Markup: unexpected option span: name={name} start={start} length={length}");
            return false;
        }

        return true;
    }
}
