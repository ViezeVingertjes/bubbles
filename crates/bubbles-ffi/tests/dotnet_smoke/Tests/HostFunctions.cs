using System.Globalization;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

internal static unsafe class HostFunctionsTest
{
    private const string Script = """
title: T
---
<<set $n = add_one(41)>>
NPC: done
===
""";

    public static unsafe int Run()
    {
        var program = Helpers.Compile(Script);
        if (program == nint.Zero) return 1;

        if (Native.bubbles_runner_new_with_saliency(program, Native.SaliencyBlrv, out var runner) != Native.Ok
            || runner == nint.Zero)
        {
            Console.Error.WriteLine($"HostFunctions: runner_new_with_saliency failed: {Helpers.LastError()}");
            return 1;
        }

        using var fnPin = new Utf8Pin("add_one");
        if (Native.bubbles_runner_register_function(runner, fnPin.Ptr, fnPin.Len, &HostAddOne, 0) != Native.Ok)
        {
            Console.Error.WriteLine($"HostFunctions: register_function failed: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (Helpers.StartNode(runner, "T") != 0)
        {
            Native.bubbles_runner_free(runner);
            return 1;
        }

        if (!Helpers.DrainAll(runner))
        {
            Console.Error.WriteLine($"HostFunctions: error draining events: {Helpers.LastError()}");
            Native.bubbles_runner_free(runner);
            return 1;
        }

        var val = Helpers.ReadVariable(runner, "$n");
        Native.bubbles_runner_free(runner);

        if (val != "{\"Number\":42.0}")
        {
            Console.Error.WriteLine($"HostFunctions: expected $n={{\"Number\":42.0}}, got {val}");
            return 1;
        }

        return 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static unsafe int HostAddOne(nint userdata, nint argsPtr, nuint argsLen, nint* outResult)
    {
        try
        {
            var span    = new ReadOnlySpan<byte>((void*)argsPtr, (int)argsLen);
            var jsonText = Encoding.UTF8.GetString(span);
            using var doc = JsonDocument.Parse(jsonText);
            var first = doc.RootElement[0];
            double n = 0;
            if (first.ValueKind == JsonValueKind.Number)
                n = first.GetDouble();
            else if (first.ValueKind == JsonValueKind.Object
                     && first.TryGetProperty("Number", out var numEl))
                n = numEl.GetDouble();

            var s     = (n + 1.0).ToString(CultureInfo.InvariantCulture);
            var bytes = Encoding.UTF8.GetBytes(s);
            fixed (byte* bp = bytes)
            {
                var p = Native.bubbles_copy_utf8((nint)bp, (nuint)bytes.Length);
                if (p == nint.Zero) return Native.Err;
                *outResult = p;
            }
            return Native.Ok;
        }
        catch
        {
            return Native.Err;
        }
    }
}
